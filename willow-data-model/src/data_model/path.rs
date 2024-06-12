#[derive(Debug)]
pub enum InvalidPathError {
    PathTooLong,
    TooManyComponents,
}

pub trait Path: PartialEq + Eq + PartialOrd + Ord + Clone {
    type Component: Eq + AsRef<[u8]> + Clone + PartialOrd + Ord;

    const MAX_COMPONENT_LENGTH: usize;
    const MAX_COMPONENT_COUNT: usize;
    const MAX_PATH_LENGTH: usize;

    fn new(components: &[Self::Component]) -> Result<Self, InvalidPathError>;

    fn empty() -> Self;

    fn append(&mut self, component: Self::Component) -> Result<(), InvalidPathError>;

    fn components(&self) -> impl Iterator<Item = &Self::Component>;

    fn prefix(&self, length: usize) -> Self {
        if length == 0 {
            return Self::empty();
        }

        if length > self.components().count() {
            return self.clone();
        }

        let mut new_path: Self = Self::empty();

        for (i, component) in self.components().enumerate() {
            new_path.append(component.clone()).unwrap();

            if i + 1 >= length {
                break;
            }
        }

        new_path
    }

    fn prefixes(&self) -> Vec<Self> {
        let self_len = self.components().count();

        (0..=self_len).map(|i| self.prefix(i)).collect()
    }

    fn is_prefix_of(&self, other: &Self) -> bool {
        let lcp = self.longest_common_prefix(other);

        lcp.components().count() == self.components().count()
    }

    fn is_prefixed_by(&self, other: &Self) -> bool {
        let lcp = self.longest_common_prefix(other);

        lcp.components().count() == other.components().count()
    }

    fn longest_common_prefix(&self, other: &Self) -> Self {
        let mut new_path = Self::empty();

        self.components()
            .zip(other.components())
            .for_each(|(a_comp, b_comp)| {
                if a_comp == b_comp {
                    new_path.append(a_comp.clone()).unwrap();
                }
            });

        new_path
    }
}

// Single threaded default.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PathComponentLocal<const MCL: usize>(Vec<u8>);

#[derive(Debug)]
pub struct ComponentTooLongError;

impl<const MCL: usize> PathComponentLocal<MCL> {
    pub fn new(bytes: &[u8]) -> Result<Self, ComponentTooLongError> {
        if bytes.len() > MCL {
            return Err(ComponentTooLongError);
        }

        let mut vec = Vec::new();
        vec.extend_from_slice(bytes);

        Ok(Self(vec))
    }
}

impl<const MCL: usize> AsRef<[u8]> for PathComponentLocal<MCL> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct PathLocal<const MCL: usize, const MCC: usize, const MPL: usize>(
    Vec<PathComponentLocal<MCL>>,
);

impl<const MCL: usize, const MCC: usize, const MPL: usize> Path for PathLocal<MCL, MCC, MPL> {
    type Component = PathComponentLocal<MCL>;

    const MAX_COMPONENT_LENGTH: usize = MCL;
    const MAX_COMPONENT_COUNT: usize = MCC;
    const MAX_PATH_LENGTH: usize = MPL;

    fn new(components: &[Self::Component]) -> Result<Self, InvalidPathError> {
        let mut path: Self = Path::empty();

        let res = components
            .iter()
            .try_for_each(|component| path.append(component.clone()));

        match res {
            Ok(_) => Ok(path),
            Err(e) => Err(e),
        }
    }

    fn empty() -> Self {
        PathLocal(Vec::new())
    }

    fn append(&mut self, component: Self::Component) -> Result<(), InvalidPathError> {
        let total_component_count = self.0.len();

        if total_component_count + 1 > MCC {
            return Err(InvalidPathError::TooManyComponents);
        }

        let total_path_length = self.0.iter().fold(0, |acc, item| acc + item.0.len());

        if total_path_length + component.as_ref().len() > MPL {
            return Err(InvalidPathError::PathTooLong);
        }

        self.0.push(component);

        Ok(())
    }

    fn components(&self) -> impl Iterator<Item = &Self::Component> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MCL: usize = 8;
    const MCC: usize = 4;
    const MPL: usize = 16;

    #[test]
    fn empty() {
        let empty_path = PathLocal::<MCL, MCC, MPL>::empty();

        assert_eq!(empty_path.components().count(), 0);
    }

    #[test]
    fn new() {
        /*
        let component_too_long = PathLocal::<MCL, MCC, MPL>::new(&[PathComponentLocal(vec![
            b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'z',
        ])]);

        assert!(matches!(component_too_long, Err(ComponentTooLongError)));
        */

        let too_many_components = PathLocal::<MCL, MCC, MPL>::new(&[
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'z']),
        ]);

        assert!(matches!(
            too_many_components,
            Err(InvalidPathError::TooManyComponents)
        ));

        let path_too_long = PathLocal::<MCL, MCC, MPL>::new(&[
            PathComponentLocal(vec![b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'a']),
            PathComponentLocal(vec![b'a', b'a', b'a', b'a', b'a', b'a', b'a', b'a']),
            PathComponentLocal(vec![b'z']),
        ]);

        assert!(matches!(path_too_long, Err(InvalidPathError::PathTooLong)));
    }

    #[test]
    fn append() {
        let mut path = PathLocal::<MCL, MCC, MPL>::empty();

        let r1 = path.append(PathComponentLocal(vec![b'a']));
        assert!(r1.is_ok());
        assert_eq!(path.components().count(), 1);
        let r2 = path.append(PathComponentLocal(vec![b'b']));
        assert!(r2.is_ok());
        assert_eq!(path.components().count(), 2);
        let r3 = path.append(PathComponentLocal(vec![b'c']));
        assert!(r3.is_ok());
        assert_eq!(path.components().count(), 3);
        let r4 = path.append(PathComponentLocal(vec![b'd']));
        assert!(r4.is_ok());
        assert_eq!(path.components().count(), 4);
        let r5 = path.append(PathComponentLocal(vec![b'z']));
        assert!(r5.is_err());

        let collected = path
            .components()
            .map(|comp| comp.as_ref())
            .collect::<Vec<&[u8]>>();

        assert_eq!(collected, vec![[b'a'], [b'b'], [b'c'], [b'd'],])
    }

    #[test]
    fn prefix() {
        let path = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
            PathComponentLocal(vec![b'c']),
        ]);

        let prefix0 = path.prefix(0);

        assert_eq!(prefix0, PathLocal::empty());

        let prefix1 = path.prefix(1);

        assert_eq!(
            prefix1,
            PathLocal::<MCL, MCC, MPL>(vec![PathComponentLocal(vec![b'a'])])
        );

        let prefix2 = path.prefix(2);

        assert_eq!(
            prefix2,
            PathLocal::<MCL, MCC, MPL>(vec![
                PathComponentLocal(vec![b'a']),
                PathComponentLocal(vec![b'b'])
            ])
        );

        let prefix3 = path.prefix(3);

        assert_eq!(
            prefix3,
            PathLocal::<MCL, MCC, MPL>(vec![
                PathComponentLocal(vec![b'a']),
                PathComponentLocal(vec![b'b']),
                PathComponentLocal(vec![b'c'])
            ])
        );

        let prefix4 = path.prefix(4);

        assert_eq!(
            prefix4,
            PathLocal::<MCL, MCC, MPL>(vec![
                PathComponentLocal(vec![b'a']),
                PathComponentLocal(vec![b'b']),
                PathComponentLocal(vec![b'c'])
            ])
        )
    }

    #[test]
    fn prefixes() {
        let path = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
            PathComponentLocal(vec![b'c']),
        ]);

        let prefixes = path.prefixes();

        assert_eq!(
            prefixes,
            vec![
                PathLocal::<MCL, MCC, MPL>(vec![]),
                PathLocal::<MCL, MCC, MPL>(vec![PathComponentLocal(vec![b'a'])]),
                PathLocal::<MCL, MCC, MPL>(vec![
                    PathComponentLocal(vec![b'a']),
                    PathComponentLocal(vec![b'b'])
                ]),
                PathLocal::<MCL, MCC, MPL>(vec![
                    PathComponentLocal(vec![b'a']),
                    PathComponentLocal(vec![b'b']),
                    PathComponentLocal(vec![b'c'])
                ]),
            ]
        )
    }

    #[test]
    fn is_prefix_of() {
        let path_a = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
        ]);

        let path_b = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
            PathComponentLocal(vec![b'c']),
        ]);

        let path_c = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'x']),
            PathComponentLocal(vec![b'y']),
            PathComponentLocal(vec![b'z']),
        ]);

        assert!(path_a.is_prefix_of(&path_b));
        assert!(!path_a.is_prefix_of(&path_c));
    }

    #[test]
    fn is_prefixed_by() {
        let path_a = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
        ]);

        let path_b = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
            PathComponentLocal(vec![b'c']),
        ]);

        let path_c = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'x']),
            PathComponentLocal(vec![b'y']),
            PathComponentLocal(vec![b'z']),
        ]);

        assert!(path_b.is_prefixed_by(&path_a));
        assert!(!path_c.is_prefixed_by(&path_a));
    }

    #[test]
    fn longest_common_prefix() {
        let path_a = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'x']),
        ]);

        let path_b = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'a']),
            PathComponentLocal(vec![b'b']),
            PathComponentLocal(vec![b'c']),
        ]);

        let path_c = PathLocal::<MCL, MCC, MPL>(vec![
            PathComponentLocal(vec![b'x']),
            PathComponentLocal(vec![b'y']),
            PathComponentLocal(vec![b'z']),
        ]);

        let lcp_a_b = path_a.longest_common_prefix(&path_b);

        assert_eq!(
            lcp_a_b,
            PathLocal::<MCL, MCC, MPL>(vec![PathComponentLocal(vec![b'a']),])
        );

        let lcp_b_a = path_b.longest_common_prefix(&path_a);

        assert_eq!(lcp_b_a, lcp_a_b);

        let lcp_a_x = path_a.longest_common_prefix(&path_c);

        assert_eq!(lcp_a_x, PathLocal::empty());
    }
}
