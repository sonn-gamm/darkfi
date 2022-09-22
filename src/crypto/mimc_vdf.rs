//! https://vitalik.ca/general/2018/07/21/starks_part_3.html

use num_bigint::BigUint;
use num_traits::Num;

/// Modulus of prime field 2^256 - 2^32 * 351 + 1
pub const MODULUS: &str =
    "115792089237316195423570985008687907853269984665640564039457584006405596119041";

/// An exponent to perform inverse of x^3 on prime field based on Fermat's Little Theorem
pub const L_FERMAT_EXPONENT: &str =
    "77194726158210796949047323339125271902179989777093709359638389337603730746027";

/// Calculates set of round constants to perform MiMC-calculation on.
fn calculate_round_constants() -> [u64; 64] {
    let mut round_constants = [0u64; 64];
    for i in 0usize..64 {
        round_constants[i] = (i.pow(7) ^ 42) as u64;
    }

    round_constants
}

/// Executes `num_steps` of MiMC-calculation in forward direction for the given `input`
fn forward_mimc(num_steps: u64, input: &BigUint) -> BigUint {
    let modulus = BigUint::from_str_radix(MODULUS, 10).unwrap();
    let round_constants = calculate_round_constants();

    let mut result = input.clone();
    let three = BigUint::from(3_u64);
    for i in 1..num_steps {
        result = (result.modpow(&three, &modulus) +
            BigUint::from(round_constants[i as usize % round_constants.len()])) %
            &modulus;
    }

    result
}

/// Executes `num_steps` of MiMC-calculation in backward direction for the given `input`.
///
/// The properties of MiMC-scheme guarantees that calculation in backward direction is
/// always slower than in forward for correctly chosen parameters.
fn backward_mimc(num_steps: u64, input: &BigUint) -> BigUint {
    let modulus = BigUint::from_str_radix(MODULUS, 10).unwrap();
    let l_fermat_exp = BigUint::from_str_radix(L_FERMAT_EXPONENT, 10).unwrap();
    let round_constants = calculate_round_constants();

    let mut result = input.clone();
    for i in (1..num_steps).rev() {
        let round_constant = BigUint::from(round_constants[i as usize % round_constants.len()]);
        result = BigUint::from(&result - &round_constant).modpow(&l_fermat_exp, &modulus);
    }

    result
}

/// Performs an Eval() step of the MiMC-based VDF
pub fn eval(seed: &BigUint, num_steps: u64) -> BigUint {
    let witness = backward_mimc(num_steps, &seed);

    witness
}

/// Performs a Verify() step for the MiMC-based VDF result
pub fn verify(seed: &BigUint, num_steps: u64, witness: &BigUint) -> bool {
    let result = forward_mimc(num_steps, witness);

    result == *seed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mimc_vdf_eval_and_verify() {
        let steps = 1000;
        let challenge = blake3::hash(b"69420").to_hex();
        let challenge = BigUint::from_str_radix(&challenge, 16).unwrap();

        let witness = eval(&challenge, steps);
        assert!(verify(&challenge, steps, &witness));
        assert_eq!(false, verify(&(&challenge - 1_u64), steps, &witness));
        assert_eq!(false, verify(&challenge, steps - 1, &witness));
        assert_eq!(false, verify(&challenge, steps, &(&witness - 1_u64)));
    }
}