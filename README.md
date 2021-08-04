# CompressedVec
An up to 4x compressed `Vec<u32>`

# Example
```rust
use compressed_vec::CVec;

fn main(){
  let mut cvec = CVec::new();

  // Like a normal Vec<u32>
  cvec.push(302);  
  cvec.get(0);

  // Compare `cvec` with normal Vec<u32> or a slice
  assert_eq!(cvec, vec![302]);
  assert_eq!(cvec, &[302]);

  cvec.pop();

  // Collect it into a u32
  let vec = ccvec.into_iter().collect::<Vec<_>>();

  // ...


  // **
  // Additional/differing functions
  // **
  
  // Returns the raw (compressed) bytes of the vector, not of the values
  cvec.as_bytes()

  // Build a new CVec from `cvec.as_bytes()` generated data
  CVec::from_bytes(..)

  // Returns the amount of bytes used to store the vectors data (compressed)
  cvec.byte_len()


  // Note: for sequencial iterating you should use a `BufCVec` or a `BufCVecRef`. This is much more efficient than iterating over `cvec.get(pos)`
  let mut buffered = BufCVecRef::from(&cvec);

}

```
