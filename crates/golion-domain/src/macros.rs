#[macro_export]
macro_rules! serde_type {
    ($item:item) => {
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $item
    };
}
