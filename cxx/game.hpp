#pragma once
#include <map>
#include <unordered_map>
#include "state.hpp"

class Game {
    struct StateInfo {
        State state;
        uint8_t depth;
        uint8_t min_distance;
        uint8_t max_distance;
        size_t left_move;
        size_t right_move;
        std::vector<std::vector<size_t>> interval_moves;
        StateInfo(const State& state, uint8_t depth) :
            state(state),
            depth(depth),
            min_distance(1),
            max_distance(255),
            left_move((size_t)-1),
            right_move((size_t)-1),
            interval_moves(0) {}
    };
    private:
        uint8_t max_clique;
        uint8_t max_colors;
        uint8_t max_intervals;
        uint8_t max_depth;
        State start_state;


        std::unordered_map<StateCode, size_t> known_states;
        std::vector<StateInfo> states;
        size_t allocate_state(const State& state, uint8_t depth);
        void visit_state_0(size_t state_id);
        void visit_state_1(size_t state_id);
        bool relax_min_distance(size_t state_id, uint8_t min_distance, uint8_t step =0);
        bool relax_max_distance(size_t state_id, uint8_t max_distance, uint8_t step =0);
        bool relax_state(size_t state_id);
        void relax_all_states();
    public:
        Game(uint8_t max_clique, uint8_t max_colors, uint8_t max_intervals, uint8_t max_depth, const State& start_state) :
            max_clique(max_clique),
            max_colors(max_colors),
            max_intervals(max_intervals),
            max_depth(max_depth),
            start_state(start_state) {}
        unsigned solve();
        void ostream(std::ostream& stream) const;
};

std::ostream& operator<< (std::ostream& stream, const Game& value);
