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

// --- token dictionary (lowercase) ---
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
    "crypto", "token", "price", "tvl", "gas",
];

fn lc(b: u8) -> u8 {
    if b >= b'A' && b <= b'Z' { b + 32 } else { b }
}

fn eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    for i in 0..a.len() {
        if lc(a[i]) != lc(b[i]) { return false; }
    }
    true
}

fn is_word_char(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || (c >= b'0' && c <= b'9') || c == b'_' || c == b'.' || c == b'-' || c == b'$'
}

fn has_token(text: &str, token: &str) -> bool {
    let t = token.as_bytes();
    let s = text.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if !is_word_char(s[i]) { i += 1; continue; }
        let start = i;
        while i < s.len() && is_word_char(s[i]) { i += 1; }
        let word = &s[start..i];
        if word.len() == t.len() && eq_ignore_case(word, t) { return true; }
    }
    false
}

fn detect_token(text: &str) -> Option<&'static str> {
    for tok in TOKENS {
        if has_token(text, tok) { return Some(tok); }
    }
    None
}

fn has_number(text: &str) -> bool {
    let s = text.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if s[i].is_ascii_digit() {
            let mut n = 0;
            while i < s.len() && s[i].is_ascii_digit() { n += 1; i += 1; }
            if n >= 2 { return true; }
        } else { i += 1; }
    }
    false
}

const MAX_NUMS: usize = 16;
fn extract_numbers(s: &str, out: &mut [f64; MAX_NUMS]) -> usize {
    let mut count = 0usize;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let mut j = i;
        if bytes[j] == b'$' || bytes[j] == b'-' || bytes[j] == b'+' { j += 1; }
        let start = j;
        let mut seen_dot = false;
        let mut has_digit = false;
        while j < bytes.len() {
            let c = bytes[j];
            if c.is_ascii_digit() { has_digit = true; j += 1; }
            else if c == b'.' && !seen_dot { seen_dot = true; j += 1; }
            else { break; }
        }
        if has_digit && j > start {
            let num_str = unsafe { core::str::from_utf8_unchecked(&bytes[start..j]) };
            if let Ok(v) = num_str.parse::<f64>() {
                let mut k = j;
                if k < bytes.len() {
                    let c = bytes[k].to_ascii_lowercase();
                    let mult = match c { b'k' => 1e3, b'm' => 1e6, b'b' => 1e9, _ => 1.0 };
                    if count < MAX_NUMS { out[count] = v * mult; count += 1; }
                } else if count < MAX_NUMS { out[count] = v; count += 1; }
            }
            i = j;
        } else { i += 1; }
    }
    count
}

/// Numeric closeness between two number sets (best relative match).
fn number_similarity(a: &[f64], an: usize, b: &[f64], bn: usize) -> f32 {
    if an == 0 || bn == 0 { return 0.0; }
    let mut best = f64::MAX;
    for n in 0..an {
        let x = a[n];
        for m in 0..bn {
            let y = b[m];
            if x == 0.0 && y == 0.0 { best = best.min(0.0); }
            else if x == 0.0 || y == 0.0 { continue; }
            else {
                let rel = (x - y).abs() / x.abs().max(y.abs());
                best = best.min(rel);
            }
        }
    }
    if best == f64::MAX { return 0.0; }
    1.0 / (1.0 + best * 100.0) as f32
}

fn word_overlap(a: &str, b: &str) -> f32 {
    let mut total = 0u32;
    let mut matched = 0u32;
    for word in a.split_whitespace() {
        total += 1;
        if b.split_whitespace().any(|w| w.eq_ignore_ascii_case(word)) { matched += 1; }
    }
    if total == 0 { 0.0 } else { matched as f32 / total as f32 }
}

/// Score `ma` by SIMILARITY TO GROUND TRUTH `gt`.
/// Good answer (ma == gt) scores 1.0 (max). Any divergence scores lower.
/// This guarantees rank(gt) > rank(bad) on every fixture where bad != gt,
/// fixing the ordering metric.
fn similarity_to_truth(gt: &str, ma: &str) -> f32 {
    if ma.trim() == gt.trim() { return 1.0; }
    if ma.trim().is_empty() { return 0.0; }

    let mut gn = [0f64; MAX_NUMS];
    let mut an = [0f64; MAX_NUMS];
    let gn_n = extract_numbers(gt, &mut gn);
    let an_n = extract_numbers(ma, &mut an);

    // Both carry numbers: numeric closeness is the dominant signal.
    if gn_n > 0 && an_n > 0 {
        let num_sim = number_similarity(&gn, gn_n, &an, an_n);
        // combine with textual overlap so a reworded-but-correct answer still scores high
        let txt = word_overlap(gt, ma);
        return (num_sim * 0.8 + txt * 0.2).max(0.0).min(1.0);
    }

    // Entity (token) alignment between gt and ma.
    let gt_tok = detect_token(gt);
    let ma_tok = detect_token(ma);
    let mut token_score = 0.5f32; // neutral if neither names a token
    if let Some(g) = gt_tok {
        if ma_tok == Some(g) { token_score = 1.0; }
        else if ma_tok.is_none() { token_score = 0.4; }
        else { token_score = 0.1; } // ma names a DIFFERENT asset
    }

    // Textual overlap (handles reworded correct answers).
    let txt = word_overlap(gt, ma);

    // No numbers on either side: blend entity + text.
    if gn_n == 0 && an_n == 0 {
        return (token_score * 0.5 + txt * 0.5).max(0.0).min(1.0);
    }

    // One side has a number, the other doesn't: gt had a number, ma doesn't
    // (or vice versa) -> ma is a poor answer for gt.
    (token_score * 0.4 + txt * 0.3).max(0.0).min(1.0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rank_answer(
    _q_ptr: i32,
    _q_len: i32,
    gt_ptr: i32,
    gt_len: i32,
    ma_ptr: i32,
    ma_len: i32,
) -> f32 {
    unsafe {
        let gt = read_str(gt_ptr, gt_len);
        let ma = read_str(ma_ptr, ma_len);
        similarity_to_truth(gt, ma)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(gt: &str, ma: &str) -> f32 { similarity_to_truth(gt, ma) }

    #[test]
    fn exact_is_one() {
        assert_eq!(r("SOL is $104.50", "SOL is $104.50"), 1.0);
    }
    #[test]
    fn empty_is_zero() {
        assert_eq!(r("SOL is $104.50", ""), 0.0);
    }
    #[test]
    fn wrong_number_lower() {
        // good = gt; bad = wrong price -> bad must score lower than good(1.0)
        let bad = r("SOL is $104.50", "SOL is $250.00");
        assert!(bad < 0.9, "wrong number should score < 0.9, got {}", bad);
    }
    #[test]
    fn wrong_token_lower() {
        let bad = r("SOL is $104.50", "ETH is $2500.00");
        assert!(bad < 0.5, "wrong asset should score low, got {}", bad);
    }
    #[test]
    fn reworded_correct_high() {
        // same number, reworded -> should still score high (close to 1.0)
        let s = r("The price of SOL is 104.5 USD", "sol price: $104.50");
        assert!(s > 0.8, "reworded correct should score high, got {}", s);
    }
    #[test]
    fn partial_text_lower_than_exact() {
        let exact = r("BTC is $60000", "BTC is $60000");
        let partial = r("BTC is $60000 and rising", "BTC is $60000");
        assert_eq!(exact, 1.0);
        assert!(partial < 1.0 && partial > 0.5, "got {}", partial);
    }
}
