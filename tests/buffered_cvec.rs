use compressed_vec::{
    buffered::{BufCVec, BufCVecRef},
    CVec,
};

#[test]
fn buf_read_seq() {
    let test_data = (0..10999).collect::<Vec<_>>();

    let mut v = CVec::new();
    for i in test_data.iter() {
        v.push(*i);
    }

    let mut buffered = BufCVec::new(v.clone());
    for (pos, i) in test_data.iter().enumerate() {
        assert_eq!(buffered.get_buffered(pos).unwrap(), i);
    }
}

#[test]
fn buf_read_spaced() {
    let test_data = (0..20999).collect::<Vec<_>>();

    let mut v = CVec::new();
    for i in test_data.iter() {
        v.push(*i);
    }

    let steps = [
        1, 10, 100, 200, 255, 256, 300, 511, 512, 513, 1022, 1024, 1025,
    ];

    for step_by in steps {
        let mut buffered = BufCVec::new(v.clone());
        for (pos, i) in test_data.iter().enumerate().step_by(step_by) {
            assert_eq!(buffered.get_buffered(pos).unwrap(), i);
        }
    }
}

#[test]
fn buf_read_seq_ref() {
    let test_data = (0..10999).collect::<Vec<_>>();

    let mut v = CVec::new();
    for i in test_data.iter() {
        v.push(*i);
    }

    let mut buffered = BufCVecRef::new(&v);
    for (pos, i) in test_data.iter().enumerate() {
        assert_eq!(buffered.get_buffered(pos).unwrap(), i);
    }
}

#[test]
fn buf_read_spaced_ref() {
    let test_data = (0..20999).collect::<Vec<_>>();

    let mut v = CVec::new();
    for i in test_data.iter() {
        v.push(*i);
    }

    let steps = [
        1, 10, 100, 200, 255, 256, 300, 511, 512, 513, 1022, 1024, 1025,
    ];

    for step_by in steps {
        let mut buffered = BufCVecRef::new(&v);
        for (pos, i) in test_data.iter().enumerate().step_by(step_by) {
            assert_eq!(buffered.get_buffered(pos).unwrap(), i);
        }
    }
}

#[test]
fn buf_read_ref_from_cvec() {
    let cvec = (0..20999).collect::<CVec>();

    let mut buffer = BufCVecRef::from(&cvec);

    assert_eq!(buffer.get_buffered(10), Some(&10));
}
