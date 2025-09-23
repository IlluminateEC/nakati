use std::sync::LazyLock;

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct TypeName(String);

impl std::fmt::Debug for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

impl std::fmt::Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for TypeName {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Monotype {
    Variable(TypeName),
    Application(Box<Monotype>, Vec<Monotype>),
}

impl std::fmt::Display for Monotype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Variable(name) => f.write_fmt(format_args!("{}", name)),
            Self::Application(function, args) => {
                if args.is_empty() {
                    return f.write_fmt(format_args!("{}", function));
                }

                f.write_str("(")?;

                f.write_str(
                    &args
                        .iter()
                        .map(|arg| arg.to_string())
                        .collect::<Vec<_>>()
                        .join(if function == &Box::new(Monotype::Variable("->".into())) {
                            " → "
                        } else {
                            " "
                        }),
                )?;

                f.write_str(")")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub enum Polytype {
    Monotype(Monotype),
    Quantifier(TypeName, Box<Polytype>),
}

impl std::fmt::Display for Polytype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Monotype(monotype) => f.write_fmt(format_args!("{}", monotype)),
            Self::Quantifier(name, body) => f.write_fmt(format_args!("(∀{}.{})", name, body)),
        }
    }
}

impl Polytype {
    pub fn from_typename(name: &str) -> Self {
        Self::Monotype(Monotype::Variable(name.into()))
    }
}

#[derive(Clone, PartialEq, PartialOrd, Ord, Eq, Debug)]
pub struct Maplet(Polytype, Polytype);

impl std::fmt::Display for Maplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} ↦ {}", self.0, self.1))
    }
}

impl Maplet {
    pub fn apply_mono(&self, mono: &Monotype) -> Option<Monotype> {
        if Polytype::Monotype(mono.clone()) == self.0 {
            Some(match &self.1 {
                Polytype::Monotype(mono) => mono.clone(),
                _ => panic!("Cannot replace a monotype with a polytype"),
            })
        } else {
            None
        }
    }

    pub fn apply(&self, poly: &Polytype) -> Option<Polytype> {
        if poly == &self.0 {
            Some(self.1.clone())
        } else {
            None
        }
    }

    pub fn map(from: &str, to: &str) -> Self {
        Self(Polytype::from_typename(from), Polytype::from_typename(to))
    }
}

#[derive(Clone, PartialOrd, Debug)]
pub struct Substitution(Vec<Maplet>);

impl std::fmt::Display for Substitution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{{{}}}",
            &self
                .0
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

impl Substitution {
    pub fn substitute_monotype(&self, mono: &Monotype) -> Monotype {
        match mono {
            Monotype::Variable(_) => {
                for maplet in &self.0 {
                    if let Some(applied) = maplet.apply_mono(mono) {
                        return applied;
                    }
                }

                return mono.clone();
            }
            Monotype::Application(function, args) => Monotype::Application(
                Box::new(self.substitute_monotype(function.as_ref())),
                args.iter()
                    .map(|monotype| self.substitute_monotype(monotype))
                    .collect(),
            ),
        }
    }

    pub fn substitute(&self, poly: &Polytype) -> Polytype {
        match poly {
            Polytype::Monotype(mono) => Polytype::Monotype(self.substitute_monotype(mono)),
            Polytype::Quantifier(forall, body) => Polytype::Quantifier(
                forall.clone(),
                Box::new(
                    self.exclude(&Polytype::Monotype(Monotype::Variable(forall.clone())))
                        .substitute(body),
                ),
            ),
        }
    }

    pub fn exclude(&self, poly: &Polytype) -> Self {
        Self(
            self.0
                .iter()
                .filter(|maplet| &maplet.0 != poly)
                .cloned()
                .collect(),
        )
    }

    /// Composes two substitutions so that it is equivalent to Self(Other(...))
    pub fn compose(&self, other: &Self) -> Self {
        let mut keys = vec![];

        self.0.iter().for_each(|key| {
            if !keys.contains(&key.0) {
                keys.push(key.0.clone())
            }
        });

        other.0.iter().for_each(|key| {
            if !keys.contains(&key.0) {
                keys.push(key.0.clone())
            }
        });

        Self(
            keys.into_iter()
                .map(|key| Maplet(key.clone(), self.substitute(&other.substitute(&key))))
                .collect(),
        )
    }
}

impl PartialEq for Substitution {
    fn eq(&self, other: &Self) -> bool {
        let mut this = self.0.clone();
        let mut that = other.0.clone();

        this.sort();
        that.sort();

        this == that
    }
}

static IDENTITY: LazyLock<Polytype> = LazyLock::new(|| {
    Polytype::Quantifier(
        "α".into(),
        Box::new(Polytype::Monotype(Monotype::Application(
            Box::new(Monotype::Variable("->".into())),
            vec![
                Monotype::Variable("α".into()),
                Monotype::Variable("α".into()),
            ],
        ))),
    )
});

#[cfg(test)]
mod tests {
    // use crate::types::Substitution;

    use super::{IDENTITY, Maplet, Monotype, Polytype, Substitution, TypeName};

    #[test]
    fn do_it() {
        panic!("{}", *IDENTITY);
    }

    #[test]
    fn example_3() {
        // αβγδε

        let s_1 = Substitution(vec![Maplet::map("α", "γ"), Maplet::map("ε", "δ")]);
        let s_2 = Substitution(vec![
            Maplet::map("α", "β"),
            Maplet::map("β", "ε"),
            Maplet::map("γ", "α"),
        ]);
        let s_3 = Substitution(vec![
            Maplet::map("α", "β"),
            Maplet::map("β", "δ"),
            Maplet::map("γ", "γ"),
            Maplet::map("ε", "δ"),
        ]);

        assert_eq!(s_1.compose(&s_2), s_3);
    }
}
