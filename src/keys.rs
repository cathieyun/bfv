use super::poly::Poly;
use super::random_source;
use rand::{CryptoRng, RngCore};

#[derive(Clone, Debug)]
pub struct SecretKey {
    pub poly: Poly,
}

#[derive(Clone, Debug)]
pub struct PublicKey {
    pub p_0: Poly,
    pub p_1: Poly,
}

#[derive(Clone, Debug)]
pub struct RelinearizationKey1 {
    pub rlk: Vec<(Poly, Poly)>, // TODO(cathie) rename this to avoid confusion.
    pub base: i64,
    pub l: i64, // TODO(cathie): change this to usize since it should never be very large.
}

#[derive(Clone, Debug)]
pub struct RelinearizationKeySimple {
    pub ek_0: Poly, // -(a * SK + e) + SK^2
    pub ek_1: Poly, // a
}

#[derive(Clone, Debug)]
pub struct RelinearizationKey2 {
    pub rlk_0: Poly,
    pub rlk_1: Poly,
    pub p: i64,
}

impl SecretKey {
    // Generate a secret key by sampling the coefficients of s uniformly
    // from R_2, which is the set {-1, 0, 1}.
    pub fn generate<T: RngCore + CryptoRng>(
        dimension: usize,
        modulus: i64,
        rng: &mut T,
    ) -> SecretKey {
        SecretKey {
            poly: random_source::get_uniform(2, dimension, modulus, rng),
        }
    }

    pub fn public_key_gen<T: RngCore + CryptoRng>(
        &self,
        q: i64,
        std_dev: f64,
        rng: &mut T,
    ) -> PublicKey {
        let s = self.poly.clone();
        let a = random_source::get_uniform(q, self.poly.dimension, q, rng);
        let e = random_source::get_gaussian(std_dev, self.poly.dimension, q, rng);

        let p_1 = a.clone();
        let p_0 = -(a.clone() * s.clone() + e);

        PublicKey { p_0, p_1 }
    }

    pub fn relinearization_key_gen_1<T: RngCore + CryptoRng>(
        &self,
        std_dev: f64,
        rng: &mut T,
    ) -> RelinearizationKey1 {
        let s = self.poly.clone();
        let q = s.q;
        // Choosing T = ceil(sqrt(q)) to minimize relinearisation time and space.
        // This can be toggled to be smaller so that the error introduced is smaller.
        // TODO(cathie): revisit this parameter / allow it to be passed in externally.
        let base = (q as f64).sqrt().ceil() as i64;

        // l is the number of levels to decompose s^2 and c_2 into.
        // l is a function of base (T in the paper): l = floor(log_T(q)).
        let l = (q as f64).log(base as f64).floor() as i64;

        let rlk = (0..l)
            .map(|i| {
                let a_i = random_source::get_uniform(q, self.poly.dimension, q, rng);
                let e_i = random_source::get_gaussian(std_dev, self.poly.dimension, q, rng);
                let base_i = base.pow(i as u32);
                let rlk_i = -(a_i.clone() * s.clone() + e_i) + (s.clone() * s.clone() * base_i);
                (rlk_i, a_i)
            })
            .collect();
        RelinearizationKey1 { rlk, base, l }
    }

    pub fn relinearization_key_gen_simple<T: RngCore + CryptoRng>(
        &self,
        std_dev: f64,
        rng: &mut T,
    ) -> RelinearizationKeySimple {
        let s = self.poly.clone();
        let q = s.q;

        let a = random_source::get_uniform(q, self.poly.dimension, q, rng);
        let e = random_source::get_gaussian(std_dev, self.poly.dimension, q, rng);
        let ek_0 = -(a.clone() * s.clone() + e) + s.clone() * s.clone();

        RelinearizationKeySimple { 
            ek_0, 
            ek_1: a,
        }
    }

    pub fn relinearization_key_gen_2<T: RngCore + CryptoRng>(
        &self,
        std_dev: f64,
        rng: &mut T,
        p: i64,
    ) -> RelinearizationKey2 {
        let q = self.poly.q;
        let mut s = self.poly.clone();
        // Change the modulus of all polynomials to p*q
        s.q = p * q;

        let a = random_source::get_uniform(p * q, self.poly.dimension, p * q, rng);
        let e = random_source::get_gaussian(std_dev, self.poly.dimension, p * q, rng);
        let rlk_0 = -(a.clone() * s.clone() + e) + s.clone() * s.clone() * p;

        RelinearizationKey2 {
            rlk_0,
            rlk_1: a,
            p,
        }
    }
}
