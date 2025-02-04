#pragma once
#include <vector>
#include <boost/functional/hash.hpp>
#include "common.hpp"

struct StateCode {
    uint64_t interval_code;
    uint64_t color_code;
    StateCode(uint64_t interval_code =0, uint64_t color_code =0) :
        interval_code(interval_code),
        color_code(color_code) {}
    bool operator< (const StateCode& other) const {
        return (interval_code < other.interval_code) or ((interval_code == other.interval_code) and (color_code < other.color_code));
    }
    bool operator== (const StateCode& other) const {
        return (interval_code == other.interval_code) and (color_code == other.color_code);
    }
    void ostream(std::ostream& stream) const;
};
std::ostream& operator<< (std::ostream& stream, const StateCode& value);
namespace std {
template<> struct hash<StateCode> {
    std::size_t operator()(const StateCode& value) const {
        std::size_t result = 0;
        boost::hash_combine(result, value.interval_code);
        boost::hash_combine(result, value.color_code);
        return result;
    }
};
}

class State {
    private:
        std::vector<uint8_t> colors;
        std::vector<bool> endpoints;
        uint8_t left_barrier;
        uint8_t right_barrier;

        void normalize_colors();
        void drop_intervals();
    public:
        State() : colors(0), endpoints(0), left_barrier(0), right_barrier(0) {};
        bool check() const;
        uint8_t get_left_barrier() const;
        uint8_t get_right_barrier() const;
        uint8_t get_interval_count() const;
        uint8_t get_interval_color(uint8_t i) const;
        std::vector<bool> get_endpoints() const;
        void add_interval(uint8_t left_endpoint, uint8_t right_endpoint, uint8_t color);
        void add_interval(const ColoredInterval cint);
        std::vector<Interval> get_intervals() const;
        std::vector<ColoredInterval> get_colored_intervals() const;
        std::vector<uint8_t> get_clique_sizes() const;
        uint8_t get_max_clique_size() const;
        uint8_t get_color_count() const;
        std::vector<uint8_t> get_started() const;
        std::vector<uint8_t> get_finished() const;
        std::vector<std::pair<Interval,std::vector<uint8_t>>> get_possible_additions(uint8_t max_clique, uint8_t max_colors) const;
        void reverse();
        void push_left_barrier();
        void push_right_barrier();
        StateCode encode() const;
        void decode(StateCode code);
        std::string dump() const;
        void load(const std::string& code);
        void ostream(std::ostream& stream) const;
};
std::ostream& operator<< (std::ostream& stream, const State& value);
