use compressed_vec::CVec;

#[test]
fn push_with_capacity() {
    let test_data = (0..9000).collect::<Vec<_>>();

    let mut v = CVec::with_capacity(10000);
    for i in test_data.iter() {
        v.push(*i);
    }
    assert_eq!(v.len(), test_data.len());

    for (pos, i) in test_data.iter().enumerate() {
        assert_eq!(v.get(pos).unwrap(), *i);
    }

    assert_eq!(v, test_data);
}

#[test]
fn test_set() {
    let mut cvec = (0..9000).collect::<CVec>();

    for i in 0..9000u32 {
        let res = cvec.get(i as usize);
        assert_eq!(res, Some(i));
    }

    cvec.set(0, 100);
    //cvec[0] = 100;
    assert_eq!(cvec.get(0), Some(100));
    for i in 1..9000u32 {
        let res = cvec.get(i as usize);
        assert_eq!(res, Some(i));
    }

    for i in 0..1000u32 {
        let res = cvec.set(i as usize, i + 1);
        assert!(res.is_some());
    }

    for i in 0..1000u32 {
        let res = cvec.get(i as usize);
        assert_eq!(res, Some(i + 1));
    }
}

#[test]
fn pop_with_capacity() {
    let mut v = CVec::with_capacity(1000);
    let mut rv = Vec::new();
    let test_data = (0..20).collect::<Vec<_>>();

    for (pos, i) in test_data.iter().enumerate() {
        v.push(*i);
        rv.push(*i);

        if pos % 2 == 0 {
            v.pop();
            rv.pop();
        }
    }

    let new_len = test_data.len() / 2;

    assert!(v.len() == new_len);
    assert!(rv.len() == v.len());
    assert_eq!(v, rv);

    for _ in 0..new_len {
        assert_eq!(v.pop(), rv.pop());
    }

    let test_data = (0..4999).collect::<Vec<_>>();

    let mut v = CVec::new();
    for i in test_data.iter() {
        v.push(*i);
    }

    for i in test_data.iter().rev() {
        assert_eq!(v.pop().unwrap(), *i);
    }
}

#[test]
fn push() {
    let test_data = (0..4999).collect::<Vec<_>>();

    let mut v = CVec::new();
    for i in test_data.iter() {
        v.push(*i);
    }
    assert_eq!(v.len(), test_data.len());
    assert_eq!(v, test_data);

    for (pos, i) in test_data.iter().enumerate() {
        assert_eq!(v.get(pos).unwrap(), *i);
    }
}

#[test]
fn pop_simple() {
    let mut v = CVec::new();
    let test_data = (0..1024).collect::<Vec<_>>();
    for i in test_data.iter() {
        v.push(*i);
    }

    for i in test_data.iter().rev() {
        assert_eq!(v.pop().unwrap(), *i);
    }
}

#[test]
fn pop() {
    let mut v = CVec::new();
    let mut rv = Vec::new();
    let test_data = (0..20).collect::<Vec<_>>();

    for (pos, i) in test_data.iter().enumerate() {
        v.push(*i);
        rv.push(*i);

        if pos % 2 == 0 {
            v.pop();
            rv.pop();
        }
    }

    let new_len = test_data.len() / 2;

    assert!(v.len() == new_len);
    assert!(rv.len() == v.len());
    assert_eq!(v, rv);

    for _ in 0..new_len {
        assert_eq!(v.pop(), rv.pop());
    }

    assert!(rv.is_empty());
    assert!(v.is_empty());
    assert_eq!(v, rv);
}

#[test]
fn capacity() {
    let v = CVec::with_capacity(1000);
    assert_eq!(v.capacity(), 1024);
}

#[test]
fn iterator() {
    let mut v = CVec::new();
    let test_data = (0..4).collect::<Vec<_>>();
    for i in test_data.iter() {
        v.push(*i);
    }

    let mut iter = v.into_iter();

    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);
}

#[test]
fn iter() {
    let mut v = CVec::new();
    let test_data = (0..4).collect::<Vec<_>>();
    for i in test_data.iter() {
        v.push(*i);
    }

    let mut iter = v.iter();

    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);
}

#[test]
fn from_iter() {
    let inp = (0..10).into_iter();

    let collected = inp.clone().collect::<CVec>();

    for (got, exp) in collected.into_iter().zip(inp.into_iter()) {
        assert_eq!(got, exp);
    }
}

#[test]
fn cmp_vec() {
    let inp = (0..10).into_iter();

    let vec = inp.clone().collect::<Vec<_>>();
    let cvec = inp.collect::<CVec>();

    assert_eq!(vec, cvec);
    assert_eq!(cvec, vec);
    assert_eq!(cvec, &vec[..]);
    assert_ne!(cvec, &vec[3..]);
}

#[test]
fn type_conversion_vec() {
    let data = 10..1000;

    // Iter -> CVec
    let cvec = data.clone().collect::<CVec>();

    // CVec -> Vec
    let vec = cvec.into_iter().collect::<Vec<_>>();

    assert_eq!(vec, data.collect::<Vec<_>>());
}

#[test]
fn extend() {
    for i in (0..577).into_iter().step_by(64) {
        for j in (0..577).into_iter().step_by(64) {
            extend_test(i, j);
        }
    }
}

fn extend_test(a_len: usize, b_len: usize) {
    let cvec_a = (0..a_len as u32).collect::<CVec>();
    let cvec_b = (0..b_len as u32).collect::<CVec>();

    let vec_a = (0..a_len as u32).collect::<Vec<_>>();
    let vec_b = (0..b_len as u32).collect::<Vec<_>>();

    let mut new_vec = cvec_a;
    new_vec.extend(cvec_b.into_iter());

    let real_vec = vec_a
        .into_iter()
        .chain(vec_b.into_iter())
        .collect::<Vec<_>>();

    for i in 0..real_vec.len() {
        let expected = new_vec.get(i);
        let real = real_vec.get(i).map(|i| *i);
        assert_eq!(expected, real);
    }
}
