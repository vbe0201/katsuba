macro_rules! check_recursion {
    (let $new_this:ident = $this:ident $($body:tt)*) => {
        $this.de.options.recursion_limit -= 1;
        if $this.de.options.recursion_limit == 0 {
            bail!("deserializer recursion limit exceeded");
        }

        let $new_this = $this $($body)*

        $new_this.de.options.recursion_limit += 1;
    };
}
