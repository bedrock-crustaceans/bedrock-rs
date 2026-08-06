use bedrock_level::types::ItemStack;
use facet::Facet;

// The records below deliberately accept unknown keys: their shapes are only
// partly known, and nbtx 4.0 rejects an unrecognised key by default.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(nbtx::allow_unknown_fields)]
pub struct BookPage {
    #[facet(rename = "photoname")]
    pub photo_name: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(nbtx::allow_unknown_fields)]
pub struct BookMetadata {
    pub author: String,
    pub xuid: String,
    pub title: String,
    pub pages: Vec<BookPage>,
    pub generation: i32,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase", nbtx::allow_unknown_fields)]
pub struct Book {
    pub count: i8,
    pub name: String,
    pub damage: i16,
    #[facet(rename = "tag")]
    pub meta: BookMetadata,
    pub was_picked_up: bool,
}

impl From<Book> for ItemStack {
    fn from(value: Book) -> Self {
        ItemStack {
            was_picked_up: value.was_picked_up,
            block: None,
            can_destroy: None,
            can_place_on: None,
            count: value.count,
            damage: value.damage,
            name: value.name,
            tag: Some(todo!("carry the book metadata across as a tag")),
        }
    }
}

impl TryFrom<ItemStack> for Book {
    type Error = ();

    fn try_from(value: ItemStack) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// The book a lectern is holding, as its four keys read together.
///
/// Not a field type: a lectern stores these keys directly in its own compound,
/// so [`Lectern`] holds them inline and [`Lectern::book`] assembles this view.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase", nbtx::allow_unknown_fields)]
pub struct LecternBook {
    pub total_pages: i32,
    pub page: i32,
    pub has_book: bool,
    pub book: Book,
}

/// A lectern, which holds a book's keys in its own compound or none at all.
///
/// serde expressed that with `#[serde(flatten)] book: Option<LecternBook>`;
/// facet has no `flatten` that nbtx honours, so the four keys are inlined as
/// optional fields and [`Self::book`] reassembles them. An empty lectern has
/// none of them, which is why every one is an `Option`.
#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct Lectern {
    pub total_pages: Option<i32>,
    pub page: Option<i32>,
    pub has_book: Option<bool>,
    pub book: Option<Book>,
}

impl Lectern {
    /// The book on this lectern, or `None` if any of its keys is missing.
    pub fn book(&self) -> Option<LecternBook> {
        Some(LecternBook {
            total_pages: self.total_pages?,
            page: self.page?,
            has_book: self.has_book?,
            book: self.book.clone()?,
        })
    }

    /// Builds a lectern holding `book`, or an empty one for `None`.
    pub fn from_book(book: Option<LecternBook>) -> Self {
        match book {
            Some(book) => Self {
                total_pages: Some(book.total_pages),
                page: Some(book.page),
                has_book: Some(book.has_book),
                book: Some(book.book),
            },
            None => Self {
                total_pages: None,
                page: None,
                has_book: None,
                book: None,
            },
        }
    }
}
