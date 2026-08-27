#pragma once

#include "ct/display_format.hpp"

#include <cstddef>
#include <functional>
#include <initializer_list>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace ct {

enum class DoorProfile {
   Iso646,
   Iso646Color,
   Cp437,
   Cp437Color,
};

enum class DoorTextRole {
   Normal,
   Heading,
   Accent,
   Prompt,
   PagerPrompt,
   Error,
   Muted,
   Label,
   Value,
   Number,
   Identifier,
   Information,
   Success,
   Warning,
   PriceFavorable,
   PriceMiddling,
   PriceUnfavorable,
   PriceMarkerFavorable,
   PriceMarkerMiddling,
   PriceMarkerUnfavorable,
};

enum class PricePlotBand {
   None,
   Favorable,
   Middling,
   Unfavorable,
};

struct PricePlotSpan {
   std::string text;
   PricePlotBand band = PricePlotBand::None;
   bool current_marker = false;
};

struct DoorTextSpan {
   std::string_view text;
   DoorTextRole role = DoorTextRole::Normal;
};

DoorProfile parse_door_profile(std::string_view text);
const char* door_profile_name(DoorProfile profile);
DoorProfile door_profile_for_capabilities(bool ansi, bool eight_bit) noexcept;
bool door_profile_uses_ansi(DoorProfile profile) noexcept;
bool door_profile_uses_cp437(DoorProfile profile) noexcept;
std::string door_single_line_field(std::string_view text);
std::string door_plain_markdown(std::string_view text);
std::string door_option_prompt(
   std::span<const std::string_view> options,
   size_t columns,
   bool leading_newline = true);
std::string door_option_prompt(
   std::initializer_list<std::string_view> options,
   size_t columns,
   bool leading_newline = true);
std::string price_box_plot(uint64_t minimum,
                           uint64_t lower_quartile,
                           uint64_t median,
                           uint64_t upper_quartile,
                           uint64_t maximum,
                           uint64_t current,
                           size_t width = 21);
std::vector<PricePlotSpan> styled_price_box_plot(
   uint64_t minimum,
   uint64_t lower_quartile,
   uint64_t median,
   uint64_t upper_quartile,
   uint64_t maximum,
   uint64_t current,
   bool buying,
   size_t width = 21);
std::optional<std::vector<std::string>> door_qr_code(
   std::string_view text,
   DoorProfile profile,
   size_t columns);

class DoorPresentation {
public:
   using Sink = std::function<void(std::string_view)>;
   enum class PagePauseAction {
      Continue,
      Continuous,
      Abort,
      SkipToPrompt,
   };
   using PagePause = std::function<PagePauseAction()>;

   DoorPresentation(DoorProfile profile,
                    size_t columns,
                    size_t rows,
                    Sink sink);

   DoorProfile profile() const noexcept;
   size_t columns() const noexcept;
   size_t content_columns() const noexcept;
   size_t rows() const noexcept;
   size_t page_content_rows(size_t reserved_rows) const noexcept;
   const DisplayFormatting& display_formatting() const noexcept;
   void set_display_formatting(DisplayFormatting formatting);

   void configure_paging(size_t reserved_rows, PagePause pause);
   void set_paging_enabled(bool enabled) noexcept;
   void resume_paging() noexcept;
   void suspend_paging() noexcept;
   void suppress_paging_until_input() noexcept;
   void reset_paging() noexcept;
   void reset_after_external_input() noexcept;
   void erase_prompt(size_t visible_columns);

   void clear();
   bool write(std::string_view text,
              DoorTextRole role = DoorTextRole::Normal);
   bool write_option_prompt(std::string_view text);
   bool write_hanging(std::string_view text,
                      size_t continuation_indent,
                      DoorTextRole role = DoorTextRole::Normal);
   bool write_hyperlink(std::string_view url,
                        DoorTextRole role = DoorTextRole::Value);
   bool write_qr_hyperlink(std::string_view url,
                           DoorTextRole role = DoorTextRole::Value);

   size_t display_width(std::string_view text) const;
   size_t labeled_field_column(
      std::span<const std::string_view> labels) const;
   size_t labeled_field_column(
      std::initializer_list<std::string_view> labels) const;
   bool write_labeled_field(
      std::string_view label,
      size_t label_width,
      std::span<const DoorTextSpan> value);
   bool write_labeled_field(
      std::string_view label,
      size_t label_width,
      std::initializer_list<DoorTextSpan> value);
   size_t ship_subsystem_label_column(size_t widest_label,
                                      size_t widest_status) const;
   size_t ship_subsystem_row_lines(std::string_view label,
                                   size_t label_width,
                                   std::string_view status) const;
   bool write_ship_subsystem_row(char selector,
                                 std::string_view label,
                                 size_t label_width,
                                 std::string_view status,
                                 DoorTextRole status_role);

private:
   void emit(std::string_view bytes);
   void flush();
   void emit_encoded(std::string_view bytes,
                     DoorTextRole role,
                     size_t continuation_indent);
   void newline(DoorTextRole role);
   void wrapped_newline(DoorTextRole role, size_t continuation_indent);
   void pause_at_page_boundary(DoorTextRole role);

   DoorProfile profile_;
   DisplayFormatting display_formatting_;
   size_t columns_;
   size_t rows_;
   size_t column_ = 0;
   size_t row_ = 0;
   size_t paging_content_rows_ = 0;
   PagePause page_pause_;
   bool paging_enabled_ = true;
   bool paging_active_ = false;
   bool paging_suppressed_until_input_ = false;
   bool handling_page_pause_ = false;
   bool write_aborted_ = false;
   bool skipping_to_prompt_ = false;
   Sink sink_;
   std::string pending_;
};

}  // namespace ct
