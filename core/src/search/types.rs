/// 能力特征：任何实现了这个 Trait 的数据，都可以被极速搜索
pub trait Searchable: Clone + Send + Sync + 'static {
    /// 唯一标识符的类型
    type Id: Clone + Send + Sync + 'static;

    /// 获取主键
    fn get_id(&self) -> Self::Id;

    /// 获取需要被检索的纯文本
    fn get_search_text(&self) -> String;
}
