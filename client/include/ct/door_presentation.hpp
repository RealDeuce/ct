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

   DoorPresentation(DoorProfile profile,
                    size_t columns,
                    size_t rows,
                    Sink sink);

   DoorProfile profile() const noexcept;
   size_t columns() const noexcept;
   size_t rows() const noexcept;
   size_t page_content_rows(size_t reserved_rows) const noexcept;

   void clear();
   void write(std::string_view text,
              DoorTextRole role = DoorTextRole::Normal);

private:
   void emit(std::string_view bytes);
   void flush();
   void emit_encoded(std::string_view bytes);
   void newline();

   DoorProfile profile_;
   size_t columns_;
   size_t rows_;
   size_t column_ = 0;
   Sink sink_;
   std::string pending_;
};

}  // namespace ct
