//! Checks.

pub mod arity {
    prelude! {}

    /// Fails if the input vector does not have exactly two elements.
    pub fn unary<T>(loc: Loc, vec: Vec<T>) -> Res<T> {
        let len = vec.len();
        let mut vec = vec.into_iter();
        match (vec.next(), vec.next()) {
            (Some(sub), None) => Ok(sub),
            _ => bail!( @loc => ErrorKind::arity_mismatch(len, 1) ),
        }
    }

    /// Fails if the input vector does not have exactly two elements.
    pub fn binary<T>(loc: Loc, vec: Vec<T>) -> Res<(T, T)> {
        let len = vec.len();
        let mut vec = vec.into_iter();
        match (vec.next(), vec.next(), vec.next()) {
            (Some(lft), Some(rgt), None) => Ok((lft, rgt)),
            _ => bail!( @loc => ErrorKind::arity_mismatch(len, 2) ),
        }
    }

    /// Fails if the input vector does not have exactly three elements.
    pub fn trinary<T>(loc: Loc, vec: Vec<T>) -> Res<(T, T, T)> {
        let len = vec.len();
        let mut vec = vec.into_iter();
        match (vec.next(), vec.next(), vec.next(), vec.next()) {
            (Some(sub1), Some(sub2), Some(sub3), None) => Ok((sub1, sub2, sub3)),
            _ => bail!( @loc => ErrorKind::arity_mismatch(len, 3) ),
        }
    }

    pub fn expect(loc: Loc, value: usize, expected: usize) -> Res<()> {
        if value != expected {
            bail!( @loc => ErrorKind::arity_mismatch(value, expected) )
        }
        Ok(())
    }
}

pub mod typ {
    prelude! {}

    /// Fails if `!typ.is_arith_like()`, at location `loc`.
    pub fn arith_like(loc: Loc, typ: &Typ) -> Res<()> {
        if !typ.is_arith_like() {
            bail!( @loc => ErrorKind::expected_arith_type(typ.clone()) )
        }
        Ok(())
    }

    /// Checks that `typ.eq(expected)`.
    pub fn expect(loc: Loc, typ: &Typ, expected: &Typ) -> Res<()> {
        if !typ.eq(expected) {
            bail!(@loc => ErrorKind::incompatible_types(typ.clone(), expected.clone()))
        }
        Ok(())
    }

    pub fn expect_bool(loc: Loc, typ: &Typ) -> Res<()> {
        expect(loc, typ, &Typ::bool())
    }
}
