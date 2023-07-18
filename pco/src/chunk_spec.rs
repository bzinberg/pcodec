use crate::chunk_metadata::PagingSpec;
use crate::errors::{PcoError, PcoResult};

/// A specification for how many elements there will be in each of a chunk's
/// data pages.
///
/// By default this specifies a single data page containing all the data.
/// You can also specify exact data page sizes via
/// [`.with_page_sizes`][Self::with_page_sizes].
/// Data pages must be specified up-front for each chunk for performance
/// reasons.
#[derive(Clone, Debug, Default)]
pub struct ChunkSpec {
  paging_spec: PagingSpec,
}

impl ChunkSpec {
  /// Modifies the spec to use the exact data page sizes given. These must
  /// sum to the actual number of elements to be compressed.
  ///
  /// E.g.
  /// ```
  /// use pco::wrapped::ChunkSpec;
  /// let spec = ChunkSpec::default().with_page_sizes(vec![1, 2, 3]);
  /// ```
  /// can only be used if the chunk actually contains 1+2+3=6 numbers.
  pub fn with_page_sizes(mut self, sizes: Vec<usize>) -> Self {
    self.paging_spec = PagingSpec::ExactPageSizes(sizes);
    self
  }

  pub(crate) fn page_sizes(&self, n: usize) -> PcoResult<Vec<usize>> {
    let page_sizes = match &self.paging_spec {
      PagingSpec::SinglePage => Ok(vec![n]),
      PagingSpec::ExactPageSizes(sizes) => {
        let sizes_n: usize = sizes.iter().sum();
        if sizes_n == n {
          Ok(sizes.clone())
        } else {
          Err(PcoError::invalid_argument(format!(
            "chunk spec suggests {} numbers but {} were given",
            sizes_n, n,
          )))
        }
      }
    }?;

    for &size in &page_sizes {
      if size == 0 {
        return Err(PcoError::invalid_argument(
          "cannot write data page of 0 numbers",
        ));
      }
    }

    Ok(page_sizes)
  }
}
