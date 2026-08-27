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

/// Extract all numeric values from a string, applying magnitude suffixes.
/// Returns up to MAX_NUMS values found. Handles "$104.50", "1.2k", "3.4M", "5B".
const MAX_NUMS: usize = 16;
fn extract_numbers(s: &str, out: &mut [f64; MAX_NUMS]) -> usize {
    let mut count = 0usize;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // allow an optional leading $ and sign
        let mut j = i;
        if bytes[j] == b'$' || bytes[j] == b'-' || bytes[j] == b'+' {
            j += 1;
        }
        // read digits and at most one decimal point
        let start = j;
        let mut seen_dot = false;
        let mut has_digit = false;
        while j < bytes.len() {
            let c = bytes[j];
            if c.is_ascii_digit() {
                has_digit = true;
                j += 1;
            } else if c == b'.' && !seen_dot {
                seen_dot = true;
                j += 1;
            } else {
                break;
            }
        }
        if has_digit && j > start {
            let num_str = unsafe { core::str::from_utf8_unchecked(&bytes[start..j]) };
            if let Ok(v) = num_str.parse::<f64>() {
                let mut k = j;
                if k < bytes.len() {
                    let c = bytes[k].to_ascii_lowercase();
                    let mult = match c {
                        b'k' => 1e3,
                        b'm' => 1e6,
                        b'b' => 1e9,
                        _ => 1.0,
                    };
                    if count < MAX_NUMS {
                        out[count] = v * mult;
                        count += 1;
                    }
                } else if count < MAX_NUMS {
                    out[count] = v;
                    count += 1;
                }
            }
            i = j;
        } else {
            i += 1;
        }
    }
    count
}

/// Compare two sets of numbers: best relative-closeness match across all pairs.
/// Returns a score in [0,1]: 1.0 if any pair matches within tolerance, decaying
/// toward 0 as the nearest pair diverges.
fn number_similarity(a: &[f64], a_n: usize, b: &[f64], b_n: usize) -> f32 {
    if a_n == 0 || b_n == 0 {
        return -1.0; // sentinel: no numbers in one side
    }
    let mut best = f64::MAX;
    for n in 0..a_n {
        let x = a[n];
        for m in 0..b_n {
            let y = b[m];
            if x == 0.0 && y == 0.0 {
                best = best.min(0.0);
            } else if x == 0.0 || y == 0.0 {
                continue;
            } else {
                let rel = (x - y).abs() / x.abs().max(y.abs());
                best = best.min(rel);
            }
        }
    }
    if best == f64::MAX {
        return 0.0;
    }
    // within 0.5% -> ~1.0; within 50% -> ~0.5; beyond -> approaches 0
    let score = 1.0 / (1.0 + best * 100.0);
    score as f32
}

fn word_overlap(answer: &str, ground: &str) -> f32 {
    let mut total = 0u32;
    let mut matched = 0u32;
    for word in answer.split_whitespace() {
        total += 1;
        if ground
            .split_whitespace()
            .any(|w| w.eq_ignore_ascii_case(word))
        {
            matched += 1;
        }
    }
    if total == 0 {
        0.0
    } else {
        matched as f32 / total as f32
    }
}

fn score(ground: &str, answer: &str) -> f32 {
    if answer == ground {
        return 1.0;
    }
    // Primary: numeric comparison (handles prices, TVL figures, APY).
    let mut gn = [0f64; MAX_NUMS];
    let mut an = [0f64; MAX_NUMS];
    let gn_n = extract_numbers(ground, &mut gn);
    let an_n = extract_numbers(answer, &mut an);
    let num_sim = number_similarity(&gn, gn_n, &an, an_n);
    if num_sim >= 0.0 {
        // blend numeric similarity with a small word-overlap bonus so that
        // textually-wrong-but-numerically-right still scores high, and
        // purely textual answers still get a floor.
        let words = word_overlap(answer, ground);
        let blended = num_sim * 0.85 + words * 0.15;
        return blended.min(1.0);
    }
    // No numbers on one side: fall back to pure word overlap.
    word_overlap(answer, ground)
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
        let ground = read_str(gt_ptr, gt_len);
        let answer = read_str(ma_ptr, ma_len);
        if answer.trim().is_empty() {
            return 0.0;
        }
        score(ground, answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(gt: &str, ma: &str) -> f32 {
        score(gt, ma)
    }

    #[test]
    fn perfect_match_is_one() {
        assert_eq!(r("SOL is $104.50", "SOL is $104.50"), 1.0);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(r("SOL is $104.50", ""), 0.0);
        assert_eq!(r("SOL is $104.50", "   "), 0.0);
    }

    #[test]
    fn numeric_match_high() {
        let s = r("The price of SOL is 104.5 USD", "sol price: $104.50");
        assert!(s > 0.9, "expected high numeric match, got {}", s);
    }

    #[test]
    fn numeric_mismatch_low() {
        let s = r("SOL is $104.50", "SOL is $250.00");
        assert!(s < 0.5, "expected low score for wrong price, got {}", s);
    }

    #[test]
    fn unrelated_low() {
        let s = r("SOL is $104.50", "The weather in Paris is nice today.");
        assert!(s < 0.3, "expected low for unrelated, got {}", s);
    }

    #[test]
    fn tvl_numeric() {
        let s = r("Uniswap TVL is 5.2B", "uniswap tvl 5200000000");
        assert!(s > 0.9, "expected high TVL numeric match, got {}", s);
    }
}
