impl IglooType {
    pub fn tokens(&self) -> TokenStream {
        match self {
            Self::Integer => quote! { Integer },
            Self::Real => quote! { Real },
            Self::Text => quote! { Text },
            Self::Boolean => quote! { Boolean },
            Self::Color => quote! { Color },
            Self::Date => quote! { Date },
            Self::Time => quote! { Time },
            Self::IntegerList => quote! { IntegerList },
            Self::RealList => quote! { RealList },
            Self::TextList => quote! { TextList },
            Self::BooleanList => quote! { BooleanList },
            Self::ColorList => quote! { ColorList },
            Self::DateList => quote! { DateList },
            Self::TimeList => quote! { TimeList },
        }
    }

    pub fn direct_type_tokens(&self) -> TokenStream {
        match self {
            Self::Integer => quote! { IglooInteger },
            Self::Real => quote! { IglooReal },
            Self::Text => quote! { IglooText },
            Self::Boolean => quote! { IglooBoolean },
            Self::Color => quote! { IglooColor },
            Self::Date => quote! { IglooDate },
            Self::Time => quote! { IglooTime },
            Self::IntegerList => quote! { IglooIntegerList },
            Self::RealList => quote! { IglooRealList },
            Self::TextList => quote! { IglooTextList },
            Self::BooleanList => quote! { IglooBooleanList },
            Self::ColorList => quote! { IglooColorList },
            Self::DateList => quote! { IglooDateList },
            Self::TimeList => quote! { IglooTimeList },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AggOp {
    Sum,
    Mean,
    Max,
    Min,
    Any,
    All,
}

impl AggOp {
    pub fn ident(&self) -> Ident {
        ident(match self {
            AggOp::Sum => "Sum",
            AggOp::Mean => "Mean",
            AggOp::Max => "Max",
            AggOp::Min => "Min",
            AggOp::Any => "Any",
            AggOp::All => "All",
        })
    }
}
