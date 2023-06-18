macro_rules! check_recursion {
    (let $new_de:ident = $de:ident $($body:tt)*) => {
        $de.options.recursion_limit -= 1;
        anyhow::ensure!(
            $de.options.recursion_limit > 0,
            "deserializer recursion limit exceeded"
        );

        let $new_de = $de $($body)*

        $de.options.recursion_limit += 1;
    };
}
