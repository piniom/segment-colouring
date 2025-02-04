#include "common.hpp"

void Interval::ostream(std::ostream& stream) const {
    stream << "" << (unsigned)left_endpoint << "-" << (unsigned)right_endpoint << "";
}
std::ostream& operator<< (std::ostream& stream, const Interval& value) {
    value.ostream(stream);
    return stream;
}

void ColoredInterval::ostream(std::ostream& stream) const {
    stream << "" << (unsigned)left_endpoint << "" << char('A'+color) << "" << (unsigned)right_endpoint << "";
}
std::ostream& operator<< (std::ostream& stream, const ColoredInterval& value) {
    value.ostream(stream);
    return stream;
}
