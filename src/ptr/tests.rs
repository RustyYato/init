use super::Uninit;

#[test]
fn test() {
    let mut x = [0, 1, 2, 3, 4, 5, 6];
    // SAFETY: a reference is always safe to pass to Uninit::from_raw
    let mut x = unsafe { Uninit::from_raw(&mut x[..]) };

    for (i, x) in x.iter_mut().enumerate() {
        // SAFETY: this is initialized
        assert_eq!(i, unsafe { *x.as_ptr() })
    }

    for (i, x) in x.iter_mut().enumerate().rev() {
        // SAFETY: this is initialized
        assert_eq!(i, unsafe { *x.as_ptr() })
    }

    for (i, x) in x.iter_mut().enumerate().rev().step_by(2) {
        // SAFETY: this is initialized
        assert_eq!(i, unsafe { *x.as_ptr() })
    }

    for (i, x) in x.iter_mut().enumerate().step_by(2) {
        // SAFETY: this is initialized
        assert_eq!(i, unsafe { *x.as_ptr() })
    }
}
