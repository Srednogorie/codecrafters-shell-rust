use crate::enums::SpecialTokens;

pub struct RedirectInfo<'a> {
    pub special_token: Option<SpecialTokens>,
    pub special_token_arg: Option<&'a str>,
}

pub struct ParseCommandTokens<'a> {
    pub command: &'a String,
    pub command_args: Vec<String>,
    pub special_token: Option<SpecialTokens>,
    pub special_token_arg: Option<&'a str>,
}
