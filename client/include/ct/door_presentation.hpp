#pragma once

#include <cstddef>
#include <functional>
#include <string>
#include <string_view>

namespace ct {

enum class DoorProfile {
   Iso646,
   Iso646Color,
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
std::string door_single_line_field(std::string_view text);

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
   size_t rows() const noexcept;
   size_t page_content_rows(size_t reserved_rows) const noexcept;

   void configure_paging(size_t reserved_rows, PagePause pause);
   void resume_paging() noexcept;
   void suspend_paging() noexcept;
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
   size_t columns_;
   size_t rows_;
   size_t column_ = 0;
   size_t row_ = 0;
   size_t paging_content_rows_ = 0;
   PagePause page_pause_;
   bool paging_active_ = false;
   bool handling_page_pause_ = false;
   Sink sink_;
   std::string pending_;
};

}  // namespace ct
