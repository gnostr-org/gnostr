#[macro_export]
macro_rules! _cmd_impl {
    ( @string $val:ident ) => {
        stringify!($val)
    };
    ( @string $val:literal ) => {{
        let ident_or_string_literal: &str = $val;
        ident_or_string_literal
    }};
    ( @string $val:tt ) => {
        ::std::compile_error!("Only identifiers or string literals supported");
    };
    ( @string ) => {
        None
    };

    ( @char $val:ident ) => {{
        let ident_or_char_literal = stringify!($val);
        debug_assert_eq!(
            ident_or_char_literal.len(),
            1,
            "Single-letter identifier expected, got {ident_or_char_literal}",
        );
        ident_or_char_literal.chars().next().unwrap()
    }};
    ( @char $val:literal ) => {{
        let ident_or_char_literal: char = $val;
        ident_or_char_literal
    }};
    ( @char ) => {{
        None
    }};

    (
        @cmd
        ($cmd:expr)
        --$long:ident
        $($tail:tt)*
    ) => {{
        let mut cmd = $cmd;
        let long = $crate::_cmd_impl! { @string $long };
        if cmd.get_name() == "" {
            cmd = cmd.name(long);
        }
        let cmd = $crate::_cmd_impl! {
            @cmd (cmd) $($tail)*
        };
        cmd
    }};
    (
        @cmd
        ($cmd:expr)
        --$long:literal
        $($tail:tt)*
    ) => {{
        let mut cmd = $cmd;
        let long = $crate::_cmd_impl! { @string $long };
        if cmd.get_name() == "" {
            cmd = cmd.name(long);
        }
        let cmd = $crate::_cmd_impl! {
            @cmd (cmd) $($tail)*
        };
        cmd
    }};
    (
        @cmd
        ($cmd:expr)
        -$short:ident
        $($tail:tt)*
    ) => {{
        let cmd = $cmd
            .visible_alias($crate::_cmd_impl! { @string $short });
        let cmd = $crate::_cmd_impl! {
            @cmd (cmd) $($tail)*
        };
        cmd
    }};
    (
        @cmd
        ($cmd:expr)
        -$short:literal
        $($tail:tt)*
    ) => {{
        let cmd = $cmd
            .visible_alias($crate::_cmd_impl! { @string $short });
        let cmd = $crate::_cmd_impl! {
            @cmd (cmd) $($tail)*
        };
        cmd
    }};
    (
        @cmd
        ($cmd:expr)
        ...
        $($tail:tt)*
    ) => {{
        let cmd = $crate::_cmd_impl! {
            @cmd (cmd) $($tail)*
        };
        cmd
    }};
    (
        @cmd
        ($cmd:expr)
        $about:literal
    ) => {{
        $cmd.about($about)
    }};
    (
        @cmd
        ($cmd:expr)
    ) => {{
        $cmd
    }};
}

#[macro_export]
macro_rules! cmd {
    ( $($tail:tt)+ ) => {{
        let cmd = clap::Command::default();
        let cmd = $crate::_cmd_impl! {
            @cmd (cmd) $($tail)+
        };
        debug_assert_ne!(cmd.get_name(), "", "Without a value or long flag, the `name:` prefix is required");
        cmd
    }};
}

