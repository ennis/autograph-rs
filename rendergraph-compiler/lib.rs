mod parser2;
mod syntax;

#[cfg(test)]
mod tests {
    use super::parser2;
    use super::syntax;

    /*#[test]
    fn parse_struct() {
        const SRC: &str = r#"
        struct CameraParameters {
            mat4 uViewMatrix;
            mat4 uProjMatrix;
            mat4 uViewProjMatrix;
            mat4 uInvProjMatrix;
            mat4 uPrevViewProjMatrixVelocity;
            mat4 uViewProjMatrixVelocity;
            vec2 uTAAOffset;
        };
        "#;
        let r = parser::parse_Struct(SRC);
        println!("{:?}", r.unwrap());
    }

    #[test]
    fn parse_empty_struct() {
        const SRC: &str = r#"struct ES {};"#;
        let r = parser::parse_Struct(SRC).unwrap();
        println!("{:?}", r);
    }

    #[test]
    fn parse_metadata() {
        const SRC: &str = r#"@A"#;
        const SRC_PARENS: &str = r#"@A()"#;
        const SRC_UNARY: &str = r#"@A(I)"#;
        const SRC_PARAMS: &str = r#"@A(I,I2,I3)"#;
        parser::parse_Metadata(SRC).unwrap();
        parser::parse_Metadata(SRC_PARENS).unwrap();
        parser::parse_Metadata(SRC_UNARY).unwrap();
        parser::parse_Metadata(SRC_PARAMS).unwrap();
    }

    #[test]
    fn parse_struct_metadata() {
        const SRC: &str = r#"
        @A
        @AB()
        @AC(test)
        @AD(test,ident)
        struct S {
            @AA float a;
            @AA @AB(ident) float b;
            @AB(ident) @AA float c;
            float d;
        };
        "#;
        let r = parser::parse_Struct(SRC).unwrap();
        println!("{:?}", r);
    }*/

}