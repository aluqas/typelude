//! 型レベルコレクション型と根本操作。
//!
//! 配列・マップ・ツリーなどのコレクション型プリミティブと
//! キャパビリティトレイト（Len、Head、Get、Set等）を提供する。
//! 型レベル高階 API は `typelude_std::core` のキャノニカル演算子名を使用する。
//!
//! ## 名前空間マッピング
//!
//! - `Len` -> `OpLen`（長さ）
//! - `Head` -> `OpHead`（先頭要素）
//! - `Tail` -> `OpTail`（尾部コレクション）
//! - `Get` -> `OpGet`（インデックスアクセス）
//! - `Set` -> `OpSet`（要素更新）
//! - `Concat` -> `OpConcat`（連結）
//! - `Append` -> 末尾追加
//! - `Prepend` -> 先頭追加
//! - `Map` -> `OpMap`（要素変換）
//! - `Fold` -> `OpFold`（畳み込み）

mod array;

pub use array::{TArr, TTerm};
pub use typelude_std::core::{Append, Concat, Get, Head, Len, Prepend, Set, Tail};
