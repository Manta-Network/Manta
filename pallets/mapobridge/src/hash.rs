use crate::crypto::sum_points;
use crate::crypto::G1;
use num::Integer;
use num_bigint::{BigInt, Sign};
use num_traits::{One, Zero};
use sp_io::hashing::keccak_256;
use sp_std::ops::{Add, Mul, Sub};
use sp_std::str::FromStr;
use sp_std::vec;
use sp_std::vec::Vec;

pub(crate) fn hash_to_g1(message: &Vec<u8>) -> G1 {
    let t0 = hash_to_base(message, 0x00, 0x01);
    let t1 = hash_to_base(message, 0x02, 0x03);

    let h0 = base_to_g1(&t0);
    let h1 = base_to_g1(&t1);

    let h = sum_points(&vec![h0, h1], &BigInt::from(3)).unwrap();
    assert!(
        bn256_g1_is_on_curve(&h),
        "Invalid hash point: not on elliptic curve"
    );
    assert!(
        safe_signing_point(&h),
        "Dangerous hash point: not safe for signing"
    );

    h
}

fn hash_to_base(msg: &Vec<u8>, dsp0: u8, dsp1: u8) -> BigInt {
    let prime = prime();

    let data0 = [vec![dsp0], msg.to_vec()].concat();
    let hash0 = BigInt::from_bytes_be(Sign::Plus, keccak_256(data0.as_slice()).as_slice());

    let data1 = [vec![dsp1], msg.to_vec()].concat();
    let hash1 = BigInt::from_bytes_be(Sign::Plus, keccak_256(data1.as_slice()).as_slice());

    let hash0 = hash0.mul(two_256_mod_prime());
    let hash0 = hash0.mod_floor(&prime);
    let hash1 = hash1.mod_floor(&prime);

    hash0.add(&hash1).mod_floor(&prime)
}

fn base_to_g1(t: &BigInt) -> G1 {
    let one = BigInt::one();
    let two = BigInt::from(2);
    let three = BigInt::from(3);
    let prime = prime();
    let curve_b = curve_b();

    let ap1 = t.modpow(&two, &prime);
    let ap2 = ap1.clone().add(&hash_const_4()).mod_floor(&prime);

    let alpha = ap1.clone().mul(&ap2).mod_floor(&prime);
    let alpha = invert(&alpha);

    let tmp = ap2.modpow(&three, &prime);
    let ap1 = ap1.modpow(&two, &prime);

    let x1 = hash_const_2().mul(&ap1).mod_floor(&prime);
    let x1 = x1.mul(&alpha).mod_floor(&prime);
    let x1 = neg(&x1);
    let x1 = x1.add(&hash_const_1()).mod_floor(&prime);

    let x2 = x1.clone().add(&one).mod_floor(&prime);
    let x2 = neg(&x2);

    let x3 = hash_const_3().mul(&tmp).mod_floor(&prime);
    let x3 = x3.mul(&alpha).mod_floor(&prime);
    let x3 = neg(&x3);
    let x3 = x3.add(&one).mod_floor(&prime);

    let y = x1.modpow(&three, &prime);
    let y = y.add(&curve_b).mod_floor(&prime);
    let residue1 = legendre(&y);

    let y = x2.modpow(&three, &prime);
    let y = y.add(&curve_b).mod_floor(&prime);
    let residue2 = legendre(&y);

    let i = (residue1 - 1) * (residue2 - 3) / 4 + 1;
    let x = if i == 1 {
        x1
    } else if i == 2 {
        x2
    } else {
        x3
    };

    let y = x.modpow(&three, &prime);
    let y = y.add(&curve_b).mod_floor(&prime);
    let y = sqrt(&y);

    let y_sign = sign0(t);
    let y = y.mul(&y_sign).mod_floor(&prime);

    let point = G1::from(&x, &y);
    assert!(
        bn256_g1_is_on_curve(&point),
        "Invalid point: not on elliptic curve"
    );

    point
}

fn curve_b() -> BigInt {
    BigInt::from(3)
}

fn hash_const_1() -> BigInt {
    BigInt::from_str("2203960485148121921418603742825762020974279258880205651966").unwrap()
}

fn hash_const_2() -> BigInt {
    BigInt::from_str("4407920970296243842837207485651524041948558517760411303933").unwrap()
}

fn hash_const_3() -> BigInt {
    BigInt::from_str(
        "14592161914559516814830937163504850059130874104865215775126025263096817472389",
    )
    .unwrap()
}

fn hash_const_4() -> BigInt {
    BigInt::from(4)
}

fn two_256_mod_prime() -> BigInt {
    BigInt::from_str("6350874878119819312338956282401532409788428879151445726012394534686998597021")
        .unwrap()
}

pub(crate) fn prime() -> BigInt {
    BigInt::from_str(
        "21888242871839275222246405745257275088696311157297823662689037894645226208583",
    )
    .unwrap()
}

fn p_minus_1() -> BigInt {
    BigInt::from_str(
        "21888242871839275222246405745257275088696311157297823662689037894645226208582",
    )
    .unwrap()
}

fn p_minus_2() -> BigInt {
    BigInt::from_str(
        "21888242871839275222246405745257275088696311157297823662689037894645226208581",
    )
    .unwrap()
}

fn p_minus_10_over_2() -> BigInt {
    BigInt::from_str(
        "10944121435919637611123202872628637544348155578648911831344518947322613104291",
    )
    .unwrap()
}

fn p_plus_10_over_4() -> BigInt {
    BigInt::from_str("5472060717959818805561601436314318772174077789324455915672259473661306552146")
        .unwrap()
}

fn invert(t: &BigInt) -> BigInt {
    t.modpow(&p_minus_2(), &prime())
}

fn neg(t: &BigInt) -> BigInt {
    if t.eq(&BigInt::zero()) {
        BigInt::zero()
    } else {
        prime().sub(t)
    }
}

fn legendre(t: &BigInt) -> i32 {
    let s = t.modpow(&p_minus_10_over_2(), &prime());
    if s.eq(&BigInt::zero()) {
        0
    } else if s.bit(0) {
        1
    } else {
        -1
    }
}

fn sqrt(t: &BigInt) -> BigInt {
    t.modpow(&p_plus_10_over_4(), &prime())
}

fn sign0(t: &BigInt) -> BigInt {
    if t.gt(&p_minus_10_over_2()) {
        p_minus_1()
    } else {
        BigInt::one()
    }
}

fn bn256_g1_is_on_curve(point: &G1) -> bool {
    let prime = prime();
    let x = BigInt::from_bytes_be(Sign::Plus, point.x.as_slice());
    let y = BigInt::from_bytes_be(Sign::Plus, point.y.as_slice());
    let left = y.modpow(&BigInt::from(2), &prime);
    let right = x.modpow(&BigInt::from(3), &prime);
    let right = right.add(&curve_b()).mod_floor(&prime);
    left.eq(&right)
}

fn safe_signing_point(point: &G1) -> bool {
    let x = BigInt::from_bytes_be(Sign::Plus, point.x.as_slice());

    !(x.eq(&BigInt::zero()) || x.eq(&BigInt::one()))
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tets_hash_to_g1() {
        let message = hex::decode("6162636566676869").unwrap();

        let t0 = hash_to_base(&message, 0x00, 0x01);
        assert_eq!(
            "5910564439876404922092592771818717274376490012008896503918548302548484008557",
            t0.to_str_radix(10)
        );

        let t1 = hash_to_base(&message, 0x02, 0x03);
        assert_eq!(
            "7054493174779650031422375588220690385584759055599915892200652159233194526041",
            t1.to_str_radix(10)
        );

        let h0 = base_to_g1(&t0);
        assert_eq!(
            "121985ee4f2f943a60920c2ff2490bd3b7d4d75ae45ea43748366fdae28b8a9c",
            hex::encode(h0.x)
        );
        assert_eq!(
            "06ee87ed6b6af23a4eecd68a146faddf4505323245fa28ae3abcf1e376373c2c",
            hex::encode(h0.y)
        );

        let h1 = base_to_g1(&t1);
        assert_eq!(
            "07a76e4c4a4f342dcd6913ef3e4869fee67d1059a765e88bec4739f96b7ffa62",
            hex::encode(h1.x)
        );
        assert_eq!(
            "19bd3f8fc5f0cad616861c37306973bb5ac1eaa9362489ef4ea52e6438792ec6",
            hex::encode(h1.y)
        );
    }
}
