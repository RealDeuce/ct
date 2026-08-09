#pragma once

#include "ct/display_format.hpp"

#include <cstddef>
#include <functional>
#include <initializer_list>
#include <string>
#include <string_view>

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
   Error,
   Muted,
   Label,
   Value,
   Number,
   Identifier,
   Information,
   Success,
   Warning,
};

DoorProfile parse_door_profile(std::string_view text);
const char* door_profile_name(DoorProfile profile);
DoorProfile door_profile_for_capabilities(bool ansi, bool eight_bit) noexcept;
bool door_profile_uses_ansi(DoorProfile profile) noexcept;
bool door_profile_uses_cp437(DoorProfile profile) noexcept;
std::string door_single_line_field(std::string_view text);
std::string door_plain_markdown(std::string_view text);
std::string door_option_prompt(
   std::initializer_list<std::string_view> options,
   size_t columns,
   bool leading_newline = true);

class DoorPresentation {
public:
   using Sink = std::function<void(std::string_view)>;
   using PagePause = std::function<void()>;

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
   void resume_paging() noexcept;
   void suspend_paging() noexcept;
   void suppress_paging_until_input() noexcept;
   void reset_paging() noexcept;
   void reset_after_external_input() noexcept;
   void erase_prompt(size_t visible_columns);

   void clear();
   void write(std::string_view text,
              DoorTextRole role = DoorTextRole::Normal);
   void write_hanging(std::string_view text,
                      size_t continuation_indent,
                      DoorTextRole role = DoorTextRole::Normal);

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
   bool paging_active_ = false;
   bool paging_suppressed_until_input_ = false;
   bool handling_page_pause_ = false;
   Sink sink_;
   std::string pending_;
};

}  // namespace ct
