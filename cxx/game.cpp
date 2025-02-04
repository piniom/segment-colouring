#include "game.hpp"

size_t Game::allocate_state(const State& state, uint8_t depth) {
    auto code = state.encode();
    if ( known_states.find(code) != known_states.end() )
        return known_states[code];
    size_t result = states.size();
    states.emplace_back(state, depth);
    State state2 = state;
    state2.reverse();
    known_states[state2.encode()] = result;
    known_states[code] = result;
    return result;
}
void Game::visit_state_0(size_t state_id) {
    auto depth = states[state_id].depth;
    if ( depth > max_depth )
        return;
    if ( states[state_id].state.get_left_barrier() < states[state_id].state.get_right_barrier() ) {
        State state2 = states[state_id].state;
        state2.push_left_barrier();
        states[state_id].left_move = allocate_state(state2, depth);
        state2 = states[state_id].state;
        state2.push_right_barrier();
        states[state_id].right_move = allocate_state(state2, depth);
    }
}
void Game::visit_state_1(size_t state_id) {
    auto depth = states[state_id].depth;
    if ( depth >= max_depth )
        return;
    auto left_move = states[state_id].left_move;
    auto right_move = states[state_id].right_move;
    if ( ( left_move != (size_t)-1 ) and states[left_move].max_distance != 255 )
        return;
    if ( ( right_move != (size_t)-1 ) and states[right_move].max_distance != 255 )
        return;
    if ( states[state_id].state.get_interval_count() < max_intervals ) {
        for ( const auto& move_responses : states[state_id].state.get_possible_additions(max_clique, max_colors) ) {
            auto& move = move_responses.first;
            std::vector<size_t> description;
            for ( auto& response : move_responses.second ) {
                ColoredInterval cint(move, response);
                State state2 = states[state_id].state;
                state2.add_interval(cint);
                description.emplace_back(allocate_state(state2, depth+1));
            }
            states[state_id].interval_moves.emplace_back(description);
        }
    }
}

bool Game::relax_min_distance(size_t state_id, uint8_t min_distance, uint8_t step) {
    if ( min_distance + step > states[state_id].min_distance ) {
        states[state_id].min_distance = min_distance + step;
        return true;
    }
    return false;
}
bool Game::relax_max_distance(size_t state_id, uint8_t max_distance, uint8_t step) {
    if ( max_distance != 255 and max_distance + step < states[state_id].max_distance ) {
        states[state_id].max_distance = max_distance + step;
        return true;
    }
    return false;
}   

bool Game::relax_state(size_t state_id) {
    bool result = false;
    auto& state_info = states[state_id];
    if ( state_info.left_move != (size_t)-1 )
        result = result or relax_max_distance(state_id, states[state_info.left_move].max_distance);
    if ( state_info.right_move != (size_t)-1 )
        result = result or relax_max_distance(state_id, states[state_info.right_move].max_distance);
    for ( const auto& options : state_info.interval_moves ) {
        uint8_t best_response = 1;
        for ( auto& response : options )
            best_response = std::max(best_response, states[response].max_distance);
        result = result or relax_max_distance(state_id, best_response, 1);
    }
    return result;
}

void Game::relax_all_states() {
    size_t improved = 1;
    for ( size_t i = 0 ; improved > 0 ; i++ ) {
        improved = 0;
        for ( size_t state_id = 0 ; state_id < states.size() ; state_id++ ) {
            improved += relax_state(state_id)?1:0;
        }
        std::cerr << "Iteration " << i << " improved " << improved << " values." << std::endl;
    }
}

unsigned Game::solve() {
    std::cerr << "Solving game on clique size " << (unsigned)max_clique << ", color count " << (unsigned)max_colors << ", board size " << (unsigned)max_intervals << ", and depth " << (unsigned)max_depth << "." << std::endl;
    auto start_id = allocate_state(start_state, 0);
    auto prev_id = start_id;
    for ( unsigned depth = 1 ; depth <= max_depth ; depth++ ) {
        for ( size_t state_id = prev_id ; state_id < states.size() ; state_id++ )
            visit_state_0(state_id);
        auto next_id = states.size();
        for ( size_t state_id = prev_id ; state_id < next_id ; state_id++ )
            visit_state_1(state_id);
        prev_id = next_id;
        std::cerr << " " << states.size() << " states on depths <= " << depth << std::endl;
        relax_all_states();
    }
    for ( size_t state_id = prev_id ; state_id < states.size() ; state_id++ )
        visit_state_0(state_id);
    std::cerr << "There are " << states.size() << " states in the game." << std::endl;
    relax_all_states();
    auto value = states[start_id].max_distance;
    if ( value != 255 )
        std::cerr << "Spoiler wins in " << (unsigned)value << " moves." << std::endl;
    else
        std::cerr << "Spoiler does not win the game." << std::endl;
    return value;
}

void Game::ostream(std::ostream& stream) const {
    stream << "<Game, clique<=" << (unsigned)max_clique << ", colors<=" << (unsigned)max_colors << ", intervals<=" << (unsigned)max_intervals << ", depth<=" << (unsigned)max_depth << ", states=" << states.size() << std::endl;
    for ( size_t state_id = 0 ; state_id < states.size() ; state_id++ ) {
        auto& state_info = states[state_id];
        auto& state = state_info.state;
        stream << "#" << state_id;
        stream << " ^" << (unsigned)state_info.depth;
        stream << " =" << (unsigned)state_info.max_distance;
        stream << " " << state;
        if ( state_info.left_move != (size_t)-1 )
            stream << " L->" << state_info.left_move;
        if ( state_info.right_move != (size_t)-1 )
            stream << " R->" << state_info.right_move;
        if ( state_info.interval_moves.size() ) {
            const auto& move_responses = state.get_possible_additions(max_clique, max_colors);
            for ( size_t i = 0 ; i < move_responses.size() ; i++ ) {
                auto& move = move_responses[i].first;
                auto& responses = move_responses[i].second;
                stream << " [" << move;
                for ( size_t j = 0 ; j < responses.size() ; j++ )
                    stream << " " << char('A'+responses[j]) << "->" << state_info.interval_moves[i][j];
                stream << "]";
            }
        }
        stream << std::endl;
    }
}
std::ostream& operator<< (std::ostream& stream, const Game& value) {
    value.ostream(stream);
    return stream;
}
