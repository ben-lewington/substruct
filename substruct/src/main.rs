pub use substruct_derive::SubStruct;

trait Project<Sub>
where
    Self: Sized,
{
    type Complement;

    fn deconstruct(self) -> (Sub, Self::Complement);

    fn proj(self) -> Sub {
        self.deconstruct().0
    }

    fn coproj(self) -> Self::Complement {
        self.deconstruct().1
    }
}

trait Immerse<Ambient>
where
    Self: Sized,
{
    type Complement;

    fn reconstruct(self, complement: Self::Complement) -> Ambient;
}

// #[derive(SubStruct)]
// #[parts(A, B)]
// #[allow(dead_code)]
// pub(crate) struct Foo {
//     #[parts = "a"]
//     pub a: i32,
//     #[parts = "a,b"]
//     b: i32,
//     c: i32,
// }
//
// #[derive(SubStruct)]
// #[parts(A)]
// #[allow(dead_code)]
// struct Bar {
//     a: i32,
//     b: i32,
// }

fn main() {
    // let x = Foo { a: 0, b: 0, c: 0 };
    // let y = AFoo { a: 1, b: 1 };
    // println!("{}{}", y.a, y.b);
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use substruct_derive::SubStruct;

    #[test]
    fn private_struct_no_generics() {
        #[derive(SubStruct)]
        #[parts(A)]
        struct Sut {
            #[parts = "a"]
            a: u8,
            b: u8,
        }

        let _ = Sut { a: 0, b: 0 };

        let _ = ASut { a: 0 };

        let _ = CoASut { b: 0 };

        #[derive(SubStruct)]
        #[parts(A, B)]
        pub(crate) struct Sut1 {
            #[parts = "a"]
            a: i32,
            #[parts = "a,b"]
            b: i32,
            c: i32,
        }

        let _ = Sut1 { a: 0, b: 0, c: 0 };

        let _ = ASut1 { a: 0, b: 0 };
        let _ = CoASut1 { c: 0 };

        let _ = BSut1 { b: 0 };
        let _ = CoBSut1 { a: 0, c: 0 };
    }

    #[test]
    fn private_struct_generics_without_phantom() {
        #[derive(SubStruct)]
        #[parts(A)]
        struct Sut<T> {
            a: T,
            b: u8,
        }
    }

}
