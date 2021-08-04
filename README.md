# CompressedVec
An up to 8x compressed `Vec<u32>`

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

# Benchmark
Here are the benchmark results on my hardware (Ryzen5 2600 ・32GB DDR4 2666MHz) <br>

```
push                    time:   [144.79 ns 145.74 ns 146.88 ns]
extend 100              time:   [812.73 ns 815.67 ns 820.80 ns]
extend 10k              time:   [57.372 us 57.409 us 57.449 us]
get() seq.              time:   [102.02 ns 103.92 ns 105.95 ns]
get() random            time:   [102.71 ns 104.68 ns 106.71 ns]
get() seq. buffered     time:   [714.13 ps 714.67 ps 715.19 ps]
pop                     time:   [104.60 ns 106.43 ns 108.30 ns]
```
The same benchmark but for a `std Vec<u32>`: <br>

```
push                    time:   [3.1506 ns 3.1565 ns 3.1631 ns]
extend 100              time:   [116.80 ns 118.04 ns 119.56 ns]
extend 10k              time:   [11.208 us 11.309 us 11.422 us]
get() seq.              time:   [0.0030 ps 0.0033 ps 0.0038 ps]
get() random            time:   [0.0015 ps 0.0017 ps 0.0020 ps]
pop                     time:   [564.90 ps 566.68 ps 569.01 ps]
```

As you can see in those benchmarks, this is not meant to be a replacement for `Vec<u32>`. <br>
It should be used where (memory)size matters and only if you can trade off some performance.

# Memory benefits

Iter from (0..10000) collected:
```
CVec:           15.5 kb
Vec<u32>:       39.1 kb
```

Iter from (0..9000000) collected:
```
CVec:           23.8 MB
Vec<u32>:       34.3 MB
```

10.000 times `10u32`:
```
CVec:           5 kb
Vec<u32>:       39.1 kb
```

9.000.000 times `10u32`:
```
CVec:           4.3MB
Vec<u32>:       34.3MB
```

<br>
As you can see the compression ratio is dependent on the size of the numbers. Storing 1000x 100u32 will always be smaller than 1000x 10000u32.
This means this library scales perfectly well for lots of smaller numbers which can't be stored as u8 or u16. <br>
In worst case a `CVec` holds `(len() / 256) * 9` bytes more than a `Vec<u32>`, which is very unlikely, unless you're only storing values which are around `u32::MAX` in value.
