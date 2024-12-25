pub use reif_macro::create_process;

pub struct Reif<F>
where
    F: Fn(&str) -> bool,
{
    pub process: F,
}

impl<F> Reif<F>
where
    F: Fn(&str) -> bool,
{
    #[inline]
    pub fn is_match(&self, heystack: &str) -> bool {
        (self.process)(heystack)
    }
}

#[macro_export]
macro_rules! new {
    ($lit:literal) => {{
        $crate::Reif {
            process: $crate::create_process!($lit),
        }
    }};

    ($($tt:tt)*) => {
        compile_error!("Expected String Literal")
    };
}

#[cfg(test)]
mod tests {
    #[ignore]
    #[test]
    fn reif_new() {
        let _reif = crate::new!("^(ab|cd)$");
    }
}
