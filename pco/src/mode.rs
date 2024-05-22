use std::fmt::Debug;

use crate::constants::Bitlen;
use crate::data_types::{FloatLike, Latent};

// Internally, here's how we should model each mode:
//
// Classic: The data is drawn from a smooth distribution.
//   Most natural data is like this.
//
// IntMult: The data is generated by 2 smooth distributions:
//   one whose outputs are multiplied by the base, and another whose outputs
//   are in the range [0, base). The 2nd process is often but not always
//   trivial.
//
// FloatMult: The data is generated by a smooth distribution
//   whose outputs get multiplied by the base and perturbed by floating point
//   errors.
//
// Note the differences between int mult and float mult,
// which have equivalent formulas.

/// A variation of how pco serializes and deserializes numbers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode<L: Latent> {
  /// Each number is compressed as
  /// * which bin it's in and
  /// * the offset in that bin.
  ///
  /// Formula: bin.lower + offset
  #[default]
  Classic,
  /// Each number is compressed as
  /// * which bin it's in and
  /// * the approximate offset in that bin as a multiplier of the base,
  /// * which bin the additional adjustment is in, and
  /// * the offset in that adjustment bin.
  ///
  /// Formula: (bin.lower + offset) * mode.base + adj_bin.lower + adj_bin.offset
  IntMult(L),
  /// Each number is compressed as
  /// * which bin it's in,
  /// * the approximate offset in that bin as a multiplier of the base,
  /// * which bin the additional ULPs adjustment is in, and
  /// * the offset in that adjustment bin.
  ///
  /// Formula: (bin.lower + offset) * mode.base +
  /// (adj_bin.lower + adj_bin.offset) * machine_epsilon
  FloatMult(L),
  /// This mode decomposes a float `x` with precision `p` into the pair `(y, m)`, where:
  /// * `y` is the float with precision `p - k` obtained by truncating the least-significant `k`
  ///   bits from the significand of `x`
  /// * `m` is a `k`-bit unsigned integer consisting of the bits removed to form `y`.
  ///
  /// Here `k` is a parameter of the mode, and `p` is deduced (by table lookup) from
  /// the underlying floating point type.
  ///
  /// Each number is compressed as
  /// * which bin the latent-bits representation of `y` is in,
  /// * the offset of the latent-bits representation of `y` in that bin,
  /// * which bin `m` is in, and
  /// * the offset of `m` in that bin.
  ///
  /// Formula: F::from_latent_bits(((y_bin.lower + y_bin.offset) << mode.k) + m)
  FloatDecomp(L),
}

impl<L: Latent> Mode<L> {
  pub(crate) fn n_latent_vars(&self) -> usize {
    use Mode::*;

    match self {
      Classic => 1,  // delta_order
      FloatMult(_) | IntMult(_) => 2,  // delta_order, mode.base
      FloatDecomp(_) => 2,  // delta_order, mode.k
    }
  }

  pub(crate) fn delta_order_for_latent_var(
    &self,
    latent_var_idx: usize,
    delta_order: usize,
  ) -> usize {
    use Mode::*;

    match (self, latent_var_idx) {
      (Classic, 0) | (FloatMult(_), 0) | (FloatDecomp(_), 0) | (IntMult(_), 0) => delta_order,
      (FloatMult(_), 1) | (IntMult(_), 1) => 0,  // `mode.base`
      (FloatDecomp(_), 1) => 0,  // `mode.k`
      _ => unreachable!(
        "unknown latent {:?}/{}",
        self, latent_var_idx
      ),
    }
  }

  pub(crate) fn float_mult<F: FloatLike<L = L>>(base: F) -> Self {
    Self::FloatMult(base.to_latent_ordered())
  }

  pub(crate) fn float_decomp<F: FloatLike<L = L>>(k: Bitlen) -> Self {
    Self::FloatDecomp(L::from_u64(k.into()))
  }
}
