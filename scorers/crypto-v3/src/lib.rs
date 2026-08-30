#![cfg_attr(not(test), no_std)]
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// --- bump allocator over a 1MB static heap ---
const HEAP_SIZE: usize = 1 * 1024 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];
static mut HEAP_OFFSET: usize = 0;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn alloc(size: i32) -> i32 {
    let size = size.max(0) as usize;
    unsafe {
        let aligned = (HEAP_OFFSET + 3) & !3;
        if aligned + size > HEAP_SIZE {
            HEAP_OFFSET = 0;
        } else {
            HEAP_OFFSET = aligned;
        }
        let ptr = core::ptr::addr_of_mut!(HEAP).cast::<u8>().add(HEAP_OFFSET);
        HEAP_OFFSET += size;
        ptr as i32
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(_ptr: i32, _size: i32) {}

unsafe fn read_str<'a>(ptr: i32, len: i32) -> &'a str {
    unsafe {
        let slice = core::slice::from_raw_parts(ptr as *const u8, len.max(0) as usize);
        core::str::from_utf8_unchecked(slice)
    }
}

// --- token / symbol dictionary (common crypto tickers + names) ---
// Used to detect which asset the question is about and whether the answer
// actually addresses it. A wrong-asset answer is a classic "bad" answer that
// still carries a correct-looking number.
const TOKENS: &[&str] = &[
    "btc", "bitcoin", "eth", "ethereum", "sol", "solana", "usdc", "usdt", "tether",
    "dai", "bnb", "binance", "xrp", "ripple", "ada", "cardano", "doge", "dogecoin",
    "avax", "avalanche", "matic", "polygon", "dot", "polkadot", "link", "chainlink",
    "ton", "toncoin", "shib", "shiba", "uni", "uniswap", "atom", "cosmos", "near",
    "apt", "aptos", "arb", "arbitrum", "op", "optimism", "ltc", "litecoin", "trx",
    "tron", "bonk", "jto", "jito", "wif", "pyth", "inj", "injective", "sui", "sei",
    "fil", "filecoin", "ldo", "lido", "aave", "comp", "compound", "mkr", "maker",
    "grt", "thegraph", "rndr", "render", "imx", "immutable", "sand", "mana", "axs",
    "algo", "algorand", "xlm", "stellar", "vet", "vechain", "ftm", "fantom", "cro",
    "crypto", "token", "coin", "price", "tvl", "gas",
];

// Hedge / uncertainty markers: bad answers often hedge.
const HEDGE_PENALTY: f32 = 0.30;
const QUALIFIER_PENALTY: f32 = 0.15;
const WRONG_ASSET_PENALTY: f32 = 0.30;
const NO_ASSET_PENALTY: f32 = 0.10;
const CONCISE_BONUS: f32 = 0.05;
const LONG_PENALTY: f32 = 0.10;
const NUMBER_BONUS: f32 = 0.25;
const TOKEN_BONUS: f32 = 0.20;
const VAGUE_TOKEN_BONUS: f32 = 0.05;
const BASE_SCORE: f32 = 0.52;

// Hedge / uncertainty markers: bad answers often hedge.
const HEDGES: &[&str] = &[
    "maybe", "perhaps", "possibly", "i think", "i'm not sure", "not sure",
    "could be", "might be", "approximately", "around", "roughly", "estimate",
    "unsure", "hard to say", "difficult to", "i believe", "probably",
];

// Contradiction / qualifier markers often present in wrong-but-plausible answers.
const QUALIFIERS: &[&str] = &[
    "but", "however", "although", "though", "actually", "on the other hand",
    "despite", "nevertheless", "whereas", "instead", "contrary",
];

fn lc(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' {
        b + 32
    } else {
        b
    }
}

fn eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if lc(a[i]) != lc(b[i]) {
            return false;
        }
    }
    true
}

fn has_token(text: &str, token: &str) -> bool {
    // token is lowercase ASCII; check as a whole word
    let t = token.as_bytes();
    let s = text.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if !is_word_char(s[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < s.len() && is_word_char(s[i]) {
            i += 1;
        }
        let word = &s[start..i];
        if word.len() == t.len() && eq_ignore_case(word, t) {
            return true;
        }
    }
    false
}

fn is_word_char(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'_' || c == b'.' || c == b'-' || c == b'$'
}

fn detect_token(text: &str) -> Option<&'static str> {
    for tok in TOKENS {
        if has_token(text, tok) {
            return Some(tok);
        }
    }
    None
}

fn contains_any(text: &str, words: &[&str]) -> bool {
    for w in words {
        let wl = w.as_bytes();
        let s = text.as_bytes();
        let mut i = 0;
        while i + wl.len() <= s.len() {
            if eq_ignore_case(&s[i..i + wl.len()], wl) {
                return true;
            }
            i += 1;
        }
    }
    false
}

fn has_number(text: &str) -> bool {
    let s = text.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if s[i].is_ascii_digit() {
            // require at least 2 digits to avoid stray single digits
            let mut n = 0;
            while i < s.len() && s[i].is_ascii_digit() {
                n += 1;
                i += 1;
            }
            if n >= 2 {
                return true;
            }
        } else {
            i += 1;
        }
    }
    false
}

/// Score a single answer for "good answer" quality.
/// Ground truth (gt) is always a gold answer -> should score high.
/// A bad miner answer with a correct-looking number but wrong asset / hedge /
/// contradiction should score low.
fn answer_quality(question: &str, answer: &str) -> f32 {
    if answer.trim().is_empty() {
        return 0.0;
    }
    let q_tok = detect_token(question);
    let a_tok = detect_token(answer);
    let mut score = BASE_SCORE;

    // A real answer carries a numeric value.
    if has_number(answer) {
        score += NUMBER_BONUS;
    }

    // Entity alignment: answer addresses the asset in the question.
    if let Some(qt) = q_tok {
        if a_tok == Some(qt) {
            score += TOKEN_BONUS; // correct asset
        } else if a_tok.is_none() {
            score -= NO_ASSET_PENALTY; // answer doesn't name any asset
        } else {
            score -= WRONG_ASSET_PENALTY; // answer names a DIFFERENT asset -> strong penalty
        }
    } else if a_tok.is_some() {
        score += VAGUE_TOKEN_BONUS; // generic asset mention when question is vague
    }

    // Directness: hedging is a hallmark of weak/wrong answers.
    if contains_any(answer, HEDGES) {
        score -= HEDGE_PENALTY;
    }
    // Contradiction/qualifier markers reduce confidence.
    if contains_any(answer, QUALIFIERS) {
        score -= QUALIFIER_PENALTY;
    }

    // Reward concise, single-fact answers; penalize very long rambling.
    let len = answer.trim().len();
    if len > 400 {
        score -= 0.10;
    } else if len <= 60 {
        score += 0.05; // conciseness bonus for a tight, direct answer
    }

    score.max(0.0).min(1.0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rank_answer(
    q_ptr: i32,
    q_len: i32,
    gt_ptr: i32,
    gt_len: i32,
    ma_ptr: i32,
    ma_len: i32,
) -> f32 {
    unsafe {
        let question = read_str(q_ptr, q_len);
        let gt = read_str(gt_ptr, gt_len);
        let ma = read_str(ma_ptr, ma_len);
        if ma.trim().is_empty() {
            return 0.0;
        }
        // Score the miner answer; gt is the reference gold answer and will
        // naturally score high via answer_quality. The evaluator ranks
        // gt above ma when f(gt) > f(ma); separation = f(gt) - f(ma).
        let s = answer_quality(question, ma);
        // Slightly prefer exact match to gt (handles identical-text cases).
        if ma.trim() == gt.trim() {
            return 1.0;
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(q: &str, gt: &str, ma: &str) -> (f32, f32) {
        (answer_quality(q, gt), answer_quality(q, ma))
    }

    #[test]
    fn gold_scores_high() {
        let (g, _) = r("price of SOL?", "SOL is $104.50", "SOL is $104.50");
        assert!(g > 0.9, "gold should score high, got {}", g);
    }

    #[test]
    fn wrong_asset_penalized() {
        // question asks SOL, bad answer talks about ETH with a number
        let (g, b) = r("price of SOL?", "SOL is $104.50", "ETH is $2500.00");
        assert!(g > 0.9, "gold {}", g);
        assert!(b < 0.7, "wrong-asset should be low, got {}", b);
        assert!(g - b > 0.3, "separation {}", g - b);
    }

    #[test]
    fn hedge_penalized() {
        let (g, b) = r("price of BTC?", "BTC is $60000", "BTC is maybe around $60000 I think");
        assert!(b < g - 0.2, "hedge should lower score");
    }

    #[test]
    fn empty_zero() {
        assert_eq!(answer_quality("q", ""), 0.0);
    }

    #[test]
    fn numeric_only_no_token() {
        // vague question, answer has number -> moderate
        let s = answer_quality("what is the price?", "104.50");
        assert!(s > 0.5 && s < 0.9, "got {}", s);
    }
}
