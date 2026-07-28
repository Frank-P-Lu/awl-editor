/// Declare a fieldless enum and derive its exhaustive `ALL` roster from the
/// same variant list. There is no second list for a maintainer to keep aligned.
macro_rules! enum_with_all {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident
            ),+ $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )+
        }

        impl $name {
            #[allow(dead_code)]
            $vis const ALL: [Self; enum_with_all!(@count $($variant),+)] = [
                $(Self::$variant,)+
            ];

            #[cfg(test)]
            #[allow(dead_code)]
            $vis const VARIANT_COUNT: usize = enum_with_all!(@count $($variant),+);
        }
    };

    (@count $($variant:ident),+) => {
        <[()]>::len(&[$(enum_with_all!(@unit $variant)),+])
    };

    (@unit $variant:ident) => {
        ()
    };
}

#[cfg(test)]
mod tests {
    enum_with_all! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Example {
            Alpha,
            Beta,
            Gamma,
        }
    }

    #[test]
    fn generated_roster_is_exact_nonempty_and_unique() {
        assert_eq!(Example::ALL.len(), Example::VARIANT_COUNT);
        assert!(!Example::ALL.is_empty());
        assert_eq!(
            Example::ALL,
            [Example::Alpha, Example::Beta, Example::Gamma]
        );
        for (i, left) in Example::ALL.iter().enumerate() {
            assert!(
                Example::ALL[(i + 1)..].iter().all(|right| left != right),
                "a generated roster contains each declared variant exactly once"
            );
        }
    }
}
