/// Ensure that a [ParseStream] is empty, or return a syn::Error if it's not.
#[macro_export]
macro_rules! ensure_eof {
    ($input:ident) => {
        if $input.is_empty() {
            Ok(())
        } else {
            Err(syn::Error::new($input.span(), "Unexpected token."))
        }
    };
}

/// Combine [syn::Result<T>]'s errors.
#[macro_export]
macro_rules! combine_results {
    ($($result:expr),*$(,)?) => {
        {
            let mut final_result: syn::Result<()> = Ok(());
            $(
                if let Err(err) = $result {
                    if let Err(ref mut final_result) = final_result {
                        final_result.combine(err);
                    } else {
                        final_result = Err(err);
                    }
                }
            )*
            final_result
        }
    };
}

/// ```rust,no_run
/// const fn comparer(lhs: u32, rhs: u32) -> ::core::cmp::Ordering;
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(const u32, const u32) -> Result
/// );
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(const u32, const u32) -> Option
/// );
/// // Returns the low value on not found.
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(const u32, const u32) -> usize
/// );
/// 
/// // Will panic on not found.
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(const u32, const u32) -> usize panic
/// );
/// 
/// const fn comparer(lhs: u32, rhs: u32, context: &[u32]) -> ::core::cmp::Ordering;
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(const u32, const u32, context: &[u32]) -> usize
/// );
/// 
/// const fn comparer(lhs: &u32, rhs: &u32) -> ::core::cmp::Ordering;
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(ref u32, ref u32) -> Result
/// );
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(ref u32, ref u32) -> Option
/// );
/// const_binary_search_fn!(
///     use path::to::comparer;
///     pub const fn search(ref u32, ref u32) -> usize
/// );
/// ```
#[macro_export]
macro_rules! const_binary_search_fn {
    (Ok($result:expr) for Result) => {
        Ok($result)
    };
    (Ok($result:expr) for Option) => {
        Some($result)
    };
    (Ok($result:expr) for usize) => {
        $result
    };
    (Err($lo:expr) for Result) => {
        Err($lo)
    };
    (Err($lo:expr) for Option) => {
        None
    };
    (Err($lo:expr) for usize) => {
        $lo
    };
    (Err($lo:expr) for usize panic) => {
        panic!("Not found.")
    };
    (return type for Result) => { ::core::result::Result<usize, usize> };
    (return type for Option) => { ::core::option::Option<usize> };
    (return type for usize) => { usize };
    (@reffed ref $expr:expr) => { &$expr };
    (@reffed const $expr:expr) => { $expr };
    (@reffed_ty ref $type:ty) => { &$type };
    (@reffed_ty const $type:ty) => { $type };
    (
        use $cmp:path;
        $(
            #[$attr:meta]
        )*
        $vis:vis const fn $name:ident(
            $search_ref:ident $search_ty:ty,
            $item_ref:ident $item_ty:ty
            $(, $context_name:ident: $ctx_ty:ty)?$(,)?
        ) -> $ret:ident $($not_found:ident!)?
    ) => {
        $(
            #[$attr]
        )*
        $vis const fn $name(
            find: $crate::const_binary_search_fn!(@reffed_ty $search_ref $search_ty),
            sorted: &[$item_ty],
            $($context_name: $ctx_ty,)?
        ) -> $crate::const_binary_search_fn!(return type for $ret) {
            let mut lo = 0usize;
            let mut hi = sorted.len();
            while lo < hi {
                let mid = (hi - lo) / 2 + lo;
                match $cmp(
                    find,
                    $crate::const_binary_search_fn!(@reffed $item_ref sorted[mid]),
                    $($context_name)?
                ) {
                    ::core::cmp::Ordering::Less => hi = mid,
                    ::core::cmp::Ordering::Equal => return $crate::const_binary_search_fn!(Ok(mid) for $ret),
                    ::core::cmp::Ordering::Greater => lo = mid + 1,
                }
            }
            $crate::const_binary_search_fn!(Err(lo) for $ret $($not_found)?)
        }
    };
}