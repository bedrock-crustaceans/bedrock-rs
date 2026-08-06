use facet::Facet;


#[derive(Debug, Copy, Clone, PartialEq, Eq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct EndGateway {
    pub age: i32,
    pub exit_portal: [i32; 3],
    pub end_gateway_bad_pos_checked: bool,
}
