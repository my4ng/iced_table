//! A table widget for iced
#![deny(missing_debug_implementations, missing_docs)]
pub use style::StyleSheet;
pub use table::{table, Table};

mod divider;
mod style;

pub mod table {
    //! Display rows of data into columns
    use iced_core::{Element, Length, Padding};
    use iced_widget::{column, container, row, scrollable, Space};

    use super::divider::Divider;
    use super::style;

    /// Creates a new [`Table`] with the provided [`Column`] definitions
    /// and [`Row`](Column::Row) data.
    ///
    /// `on_sync` is needed to keep the header & footer scrollables in sync with
    /// the body scrollable. It is up to the consumer to emit a [`scroll_to`](iced_widget::scrollable::scroll_to) operation
    /// from `update` when this message is received.
    pub fn table<'a, Column, Row, Message, Renderer>(
        header: scrollable::Id,
        body: scrollable::Id,
        columns: &'a [Column],
        rows: &'a [Row],
        on_sync: fn(scrollable::AbsoluteOffset) -> Message,
    ) -> Table<'a, Column, Row, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
    {
        Table {
            header,
            body,
            footer: None,
            columns,
            rows,
            on_sync,
            on_column_drag: None,
            on_column_release: None,
            min_width: 0.0,
            divider_width: 2.0,
            cell_padding: 4.into(),
            style: Default::default(),
            scrollable_properties: Box::new(Default::default),
        }
    }

    /// The type used to determine how the width of a [`Column`] should be calculated.
    #[derive(Debug, Copy, Clone)]
    pub enum Width {
        /// Fixed width; the width cannot be resized.
        Fixed(f32),
        /// Resizable width, where the current width is the sum of initial and offset.
        /// The current width can be clamped to a range by the consumer.
        Resizable {
            /// Initial width.
            initial: f32,
            /// Temporary offset when dragged.
            offset: f32,
        },
        /// Fill the remaining width of the table based on the proportion specified,
        /// shared with all other [`Column`] in the same table.
        Fill {
            /// Proprotion between all fill columns.
            proportion: u32,
            /// Minimum width (or `0.0f32` to represent no minimum).
            minimum: f32,
        },
    }

    #[derive(Debug, Clone, Copy)]
    struct CalculatedWidth {
        current: f32,
        is_resizable: bool, // only applicable to resizable widths
    }

    /// Defines what a column looks like for each [`Row`](Column::Row) of data.
    pub trait Column<'a, 'b, Message, Renderer> {
        /// A row of data.
        type Row;

        /// Define the header [`Element`] for this column.
        fn header(&'b self, col_index: usize) -> Element<'a, Message, Renderer>;

        /// Define the cell [`Element`] for this column.
        fn cell(
            &'b self,
            col_index: usize,
            row_index: usize,
            row: &'b Self::Row,
        ) -> Element<'a, Message, Renderer>;

        /// Define the footer [`Element`] for this column.
        fn footer(
            &'b self,
            _col_index: usize,
            _rows: &'b [Self::Row],
        ) -> Option<Element<'a, Message, Renderer>> {
            None
        }

        /// Return the width type for this column.
        fn width(&self) -> Width;
    }

    /// An element to display rows of data into columns.
    #[allow(missing_debug_implementations)]
    pub struct Table<'a, Column, Row, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
    {
        header: scrollable::Id,
        body: scrollable::Id,
        footer: Option<scrollable::Id>,
        columns: &'a [Column],
        rows: &'a [Row],
        on_sync: fn(scrollable::AbsoluteOffset) -> Message,
        on_column_drag: Option<fn(usize, f32) -> Message>,
        on_column_release: Option<Message>,
        min_width: f32,
        divider_width: f32,
        cell_padding: Padding,
        style: <Renderer::Theme as style::StyleSheet>::Style,
        // TODO: Upstream make this Copy
        scrollable_properties: Box<dyn Fn() -> scrollable::Properties + 'a>,
    }

    impl<'a, Column, Row, Message, Renderer> Table<'a, Column, Row, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
    {
        /// Sets the message that will be produced when a [`Column`] is resizing. Setting this
        /// will enable the resizing interaction.
        ///
        /// `on_drag` will emit a message during an on-going resize. It is up to the consumer to return
        /// this value for the associated column in [`Column::resize_offset`].
        ///
        /// `on_release` is emited when the resize is finished. It is up to the consumer to apply the last
        /// `on_drag` offset to the column's stored width.
        pub fn on_column_resize(
            self,
            on_drag: fn(usize, f32) -> Message,
            on_release: Message,
        ) -> Self {
            Self {
                on_column_drag: Some(on_drag),
                on_column_release: Some(on_release),
                ..self
            }
        }

        /// Show the footer returned by [`Column::footer`].
        pub fn footer(self, footer: scrollable::Id) -> Self {
            Self {
                footer: Some(footer),
                ..self
            }
        }

        /// Sets the minimum width of table.
        ///
        /// This is useful to use in conjuction with [`responsive`](iced_widget::responsive) to ensure
        /// the table always fills the width of it's parent container.
        pub fn min_width(self, min_width: f32) -> Self {
            Self { min_width, ..self }
        }

        /// Sets the width of the column dividers.
        pub fn divider_width(self, divider_width: f32) -> Self {
            Self {
                divider_width,
                ..self
            }
        }

        /// Sets the [`Padding`] used inside each cell of the [`Table`].
        pub fn cell_padding(self, cell_padding: impl Into<Padding>) -> Self {
            Self {
                cell_padding: cell_padding.into(),
                ..self
            }
        }

        /// Sets the style variant of this [`Table`].
        pub fn style(
            self,
            style: impl Into<<Renderer::Theme as style::StyleSheet>::Style>,
        ) -> Self {
            Self {
                style: style.into(),
                ..self
            }
        }

        ///  Sets the [`Properties`](iced_widget::scrollable::Properties) used for the table's body scrollable.
        pub fn scrollable_properties(self, f: impl Fn() -> scrollable::Properties + 'a) -> Self {
            Self {
                scrollable_properties: Box::new(f),
                ..self
            }
        }
    }

    impl<'a, 'b, Column, Row, Message, Renderer> From<Table<'b, Column, Row, Message, Renderer>>
        for Element<'a, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet + scrollable::StyleSheet,
        Column: self::Column<'a, 'b, Message, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        fn from(table: Table<'b, Column, Row, Message, Renderer>) -> Self {
            let Table {
                header,
                body,
                footer,
                columns,
                rows,
                on_sync,
                on_column_drag,
                on_column_release,
                min_width,
                divider_width,
                cell_padding,
                style,
                scrollable_properties,
            } = table;

            let (calaculated_widths, unused_width) = distribute_fill_widths(columns, min_width);

            let header = scrollable(style::wrapper::header(
                row(columns
                    .iter()
                    .zip(calaculated_widths.iter())
                    .enumerate()
                    .map(|(index, (column, &calculated_width))| {
                        header_container(
                            index,
                            column,
                            calculated_width,
                            on_column_drag,
                            on_column_release.clone(),
                            divider_width,
                            cell_padding,
                            style.clone(),
                        )
                    })
                    .collect()),
                style.clone(),
            ))
            .id(header)
            .horizontal_scroll(
                scrollable::Properties::new()
                    .width(0)
                    .margin(0)
                    .scroller_width(0),
            )
            .vertical_scroll(
                scrollable::Properties::new()
                    .width(0)
                    .margin(0)
                    .scroller_width(0),
            );

            let body = scrollable(column(
                rows.iter()
                    .enumerate()
                    .map(|(row_index, _row)| {
                        style::wrapper::row(
                            row(columns
                                .iter()
                                .zip(calaculated_widths.iter())
                                .enumerate()
                                .map(|(col_index, (column, &calculated_width))| {
                                    body_container(
                                        col_index,
                                        row_index,
                                        calculated_width,
                                        column,
                                        _row,
                                        divider_width,
                                        cell_padding,
                                    )
                                })
                                .collect()),
                            style.clone(),
                            row_index,
                        )
                    })
                    .collect(),
            ))
            .id(body)
            .on_scroll(move |viewport| {
                let offset = viewport.absolute_offset();
                (on_sync)(scrollable::AbsoluteOffset { y: 0.0, ..offset })
            })
            .horizontal_scroll((scrollable_properties)())
            .vertical_scroll((scrollable_properties)())
            .height(Length::Fill);

            let footer = footer.map(|footer| {
                scrollable(style::wrapper::footer(
                    row(columns
                        .iter()
                        .zip(calaculated_widths.iter())
                        .enumerate()
                        .map(|(index, (column, &calculated_width))| {
                            footer_container(
                                index,
                                column,
                                calculated_width,
                                rows,
                                on_column_drag,
                                on_column_release.clone(),
                                divider_width,
                                cell_padding,
                                style.clone(),
                            )
                        })
                        .collect()),
                    style,
                ))
                .id(footer)
                .horizontal_scroll(
                    scrollable::Properties::new()
                        .width(0)
                        .margin(0)
                        .scroller_width(0),
                )
                .vertical_scroll(
                    scrollable::Properties::new()
                        .width(0)
                        .margin(0)
                        .scroller_width(0),
                )
            });

            let mut column = column![header, body];

            if let Some(footer) = footer {
                column = column.push(footer);
            }

            let mut table_container = container(column).height(Length::Fill).width(Length::Shrink);

            if let Some(unused_width) = unused_width {
                table_container = table_container.padding([0.0, unused_width, 0.0, 0.0]);
            }

            table_container.into()
        }
    }

    fn header_container<'a, 'b, Column, Row, Message, Renderer>(
        index: usize,
        column: &'b Column,
        calculated_width: CalculatedWidth,
        on_drag: Option<fn(usize, f32) -> Message>,
        on_release: Option<Message>,
        divider_width: f32,
        cell_padding: Padding,
        style: <Renderer::Theme as style::StyleSheet>::Style,
    ) -> Element<'a, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
        Column: self::Column<'a, 'b, Message, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let content = container(column.header(index))
            .width(Length::Fill)
            .padding(cell_padding)
            .into();

        with_divider(
            index,
            calculated_width,
            content,
            on_drag,
            on_release,
            divider_width,
            style,
        )
    }

    fn body_container<'a, 'b, Column, Row, Message, Renderer>(
        col_index: usize,
        row_index: usize,
        calculated_width: CalculatedWidth,
        column: &'b Column,
        row: &'b Row,
        divider_width: f32,
        mut cell_padding: Padding,
    ) -> Element<'a, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
        Column: self::Column<'a, 'b, Message, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        if calculated_width.is_resizable {
            cell_padding.right += divider_width;
        }

        container(column.cell(col_index, row_index, row))
            .width(calculated_width.current)
            .padding(cell_padding)
            .into()
    }

    fn footer_container<'a, 'b, Column, Row, Message, Renderer>(
        index: usize,
        column: &'b Column,
        calculated_width: CalculatedWidth,
        rows: &'b [Row],
        on_drag: Option<fn(usize, f32) -> Message>,
        on_release: Option<Message>,
        divider_width: f32,
        cell_padding: Padding,
        style: <Renderer::Theme as style::StyleSheet>::Style,
    ) -> Element<'a, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
        Column: self::Column<'a, 'b, Message, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let content = if let Some(footer) = column.footer(index, rows) {
            container(footer)
                .width(Length::Fill)
                .padding(cell_padding)
                .center_y()
                .into()
        } else {
            Element::from(Space::with_width(Length::Fill))
        };

        with_divider(
            index,
            calculated_width,
            content,
            on_drag,
            on_release,
            divider_width,
            style,
        )
    }

    fn with_divider<'a, Message, Renderer>(
        index: usize,
        calculated_width: CalculatedWidth,
        content: Element<'a, Message, Renderer>,
        on_drag: Option<fn(usize, f32) -> Message>,
        on_release: Option<Message>,
        divider_width: f32,
        style: <Renderer::Theme as style::StyleSheet>::Style,
    ) -> Element<'a, Message, Renderer>
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
        Message: 'a + Clone,
    {
        let current = calculated_width.current;
        if let Some((on_drag, on_release)) = on_drag.zip(on_release) {
            if calculated_width.is_resizable {
                return container(Divider::new(
                    content,
                    divider_width,
                    move |offset| (on_drag)(index, offset),
                    on_release,
                    style,
                ))
                .width(current)
                .into();
            }
        }

        container(content).width(current).into()
    }

    // If there is no fill column, return `remaining_width` if positive.
    //
    // If there is at least one, then distribute the remaining width, based on their proportions,
    // to fill the remaining width.
    //
    // If there is no remaining space or if the distributed width is less than their minimum width,
    // then use the minimum instead.
    fn distribute_fill_widths<'a, 'b, Column, Row, Message, Renderer>(
        columns: &'b [Column],
        min_width: f32,
    ) -> (Vec<CalculatedWidth>, Option<f32>)
    where
        Renderer: iced_core::Renderer + 'a,
        Renderer::Theme: style::StyleSheet + container::StyleSheet,
        Column: self::Column<'a, 'b, Message, Renderer, Row = Row>,
        Message: 'a + Clone,
    {
        let mut fill_proportion = 0;
        let mut remaining_width = min_width;

        columns.iter().for_each(|column| match column.width() {
            Width::Fixed(current) => remaining_width -= current,
            Width::Resizable {
                initial, offset, ..
            } => remaining_width -= initial + offset,
            Width::Fill { proportion, .. } => fill_proportion += proportion,
        });

        // Calculate the width of a single part to avoid division for every fill column
        let part_width = if fill_proportion != 0 {
            remaining_width / fill_proportion as f32
        } else {
            0.0
        };

        let calculated_widths = columns
            .iter()
            .map(|column| match column.width() {
                Width::Fixed(current) => CalculatedWidth {
                    current,
                    is_resizable: false,
                },
                Width::Resizable { initial, offset } => CalculatedWidth {
                    current: initial + offset,
                    is_resizable: true
                },
                Width::Fill {
                    proportion,
                    minimum,
                } => CalculatedWidth {
                    current: (proportion as f32 * part_width).max(minimum),
                    is_resizable: false,
                },
            })
            .collect();

        let unused_width =
            (remaining_width > 0.0 && fill_proportion == 0).then_some(remaining_width);

        (calculated_widths, unused_width)
    }
}
