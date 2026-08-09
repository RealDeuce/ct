#include "ct/protocol.hpp"

#include <stdexcept>

namespace {

void check(const bool condition)
{
   if(!condition) {
      throw std::runtime_error("protocol test failed");
   }
}

}  // namespace

int main()
{
   check(ct::language_selection_matches("en", "en-US"));
   check(ct::language_selection_matches("EN-us", "en-US"));
   check(ct::language_selection_matches("en-US", "en"));
   check(!ct::language_selection_matches("en-US", "en-GB"));
   check(!ct::language_selection_matches("en", "de"));
   check(!ct::language_selection_matches("", "en-US"));
   check(!ct::language_selection_matches("en-US", ""));
}
