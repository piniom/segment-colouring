#pragma once
#include <cstdint>
#include <iomanip>
#include <iostream>

struct Interval {
    uint8_t left_endpoint;
    uint8_t right_endpoint;
    Interval(uint8_t left_endpoint =0, uint8_t right_endpoint =0) :
        left_endpoint(left_endpoint),
        right_endpoint(right_endpoint) {}
    bool operator==(const Interval& other) const {
        return left_endpoint == other.left_endpoint and right_endpoint == other.right_endpoint;
    }
    void ostream(std::ostream& stream) const;
};
std::ostream& operator<< (std::ostream& stream, const Interval& value);

struct ColoredInterval {
    uint8_t left_endpoint;
    uint8_t right_endpoint;
    uint8_t color;
    ColoredInterval(uint8_t left_endpoint =0, uint8_t right_endpoint =0, uint8_t color =0) :
        left_endpoint(left_endpoint),
        right_endpoint(right_endpoint),
        color(color) {}
    ColoredInterval(const Interval& interval, uint8_t color =0) :
        left_endpoint(interval.left_endpoint),
        right_endpoint(interval.right_endpoint),
        color(color) {}
    void ostream(std::ostream& stream) const;
};
std::ostream& operator<< (std::ostream& stream, const ColoredInterval& value);
