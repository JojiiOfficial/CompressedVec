# CompressedVec
An up to 4x compressed `Vec<u32>`

# Example
```rust
use compressed_vec::CVec;

fn main(){
  let mut vec = CVec::new();

  // Like a normal Vec<u32>
  vec.push(302);  
  vec.get(0);
  vec.pop();
  // ...
}

```
