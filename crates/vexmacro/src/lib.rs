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