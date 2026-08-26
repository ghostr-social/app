use super::*;

const MAX_CATEGORIES: usize = 4;

impl WatchContext {
    pub(crate) fn with_creator(mut self, creator: WatchKey) -> Self {
        self.creator = Some(creator);
        self
    }

    pub(crate) fn with_categories(
        mut self,
        categories: impl IntoIterator<Item = WatchKey>,
    ) -> Self {
        for category in categories {
            if self.categories.len() == MAX_CATEGORIES {
                break;
            }
            if !self.categories.contains(&category) {
                self.categories.push(category);
            }
        }
        self
    }

    pub(crate) fn with_user(mut self, user: WatchKey) -> Self {
        self.user = Some(user);
        self
    }
}
