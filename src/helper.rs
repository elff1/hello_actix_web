pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut source = e.source();
    while let Some(cause) = source {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        source = cause.source();
    }

    Ok(())
}
