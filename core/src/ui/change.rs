
use std::ops::Add;

use alga::general::*;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Change<A> {
    Unchanged,
    Unset,
    Set(A),
}

impl<A> Change<A> {
    pub fn map<B, F: Fn(A) -> B>(self, f: F) -> Change<B> {
        use self::Change::*;
        match self {
            Set(a) => Set(f(a)),
            Unset => Unset,
            Unchanged => Unchanged,
        }
    }
}

// impl<A: Add<Output=A>> Add for Change<A> {
impl<A: AbstractMagma<Additive>> Add for Change<A> {
    type Output = Change<A>;
    fn add(self, rhs: Change<A>) -> Self::Output {
        self.operate(&rhs)
        // use self::Change::*;
        // match (self, rhs) {
        //     (Unchanged, r) => r,
        //     (l, Unchanged) => l,
        //     (Set(a), Set(b)) => Set(a + b),
        //     (_,r) => r
        // }
    }
}

impl<A> Identity<Additive> for Change<A> {
    fn identity() -> Change<A> {
        Change::Unchanged
    }
}

impl<A: AbstractMagma<Additive>> AbstractMagma<Additive> for Change<A> {
    fn operate(&self, rhs: &Self) -> Self {
        use self::Change::*;
        match (self, rhs) {
            (Unchanged, r) => r.clone(),
            (l, Unchanged) => l.clone(),
            (Set(a), Set(b)) => Set(a.operate(b)),
            (_,r) => r.clone()
        }
    }
}

// impl<A: AbstractSemigroup<Additive>> AbstractSemigroup<Additive> for Change<A> {
// }

// impl<A: AbstractMonoid<Additive>> AbstractMonoid<Additive> for Change<A> {
// }

// impl<A: AdditiveMonoid> AdditiveMonoid for Change<A> {
// }

#[cfg(test)]
mod tests {
    use quickcheck::*;
    use rand::seq::SliceRandom;
    use super::*;

    impl<A: Arbitrary> Arbitrary for Change<A> {
        fn arbitrary<G: Gen>(g: &mut G) -> Change<A> {
            [ Change::Unchanged
            , Change::Unset
            , Change::Set(A::arbitrary(g))
            ].choose(g).unwrap().to_owned()
        }
    }

    // #[test]
    // fn prop_identity() {
    //     fn prop(args: Change<i8>) -> bool {
    //         Change::prop_operating_identity_element_is_noop((args,))
    //     }
    //     quickcheck(prop as fn(_) -> _);
    // }

    // #[test]
    // fn prop_associative() {
    //     fn prop(args: (Change<i8>, Change<i8>, Change<i8>)) -> bool {
    //         let args2 = (args.0.map(|x| x as i32), args.1.map(|x| x as i32), args.2.map(|x| x as i32));
    //         Change::prop_is_associative(args2)
    //     }
    //     quickcheck(prop as fn(_) -> _);
    // }

}

