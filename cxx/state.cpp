#include <algorithm>
#include "state.hpp"

void StateCode::ostream(std::ostream& stream) const {
    stream << interval_code << "@" << color_code;
}
std::ostream& operator<< (std::ostream& stream, const StateCode& value) {
    value.ostream(stream);
    return stream;
}

void State::normalize_colors() {
    std::vector<uint8_t> color_map;
    uint8_t next_color = 0;
    for ( uint8_t i = 0 ; i < colors.size() ; i++ ) {
        auto c = colors[i];
        if ( c >= color_map.size() )
            color_map.resize(c+1u, 255);
        if ( color_map[c] == 255 )
            color_map[c] = next_color++;
        colors[i] = color_map[c];
    }
}

void State::drop_intervals() {
    uint8_t left_counter = 0;
    uint8_t right_counter = 0;
    for ( uint8_t i = 0 ; i < left_barrier; i++ )
        if ( endpoints[i] != true )
            left_counter++;
    for ( uint8_t i = right_barrier; i < endpoints.size() ; i++ )
        if ( endpoints[i] != false )
            right_counter++;
    if ( left_counter > 0 )
        colors = std::vector(colors.begin()+left_counter, colors.begin()+colors.size()-right_counter);
    else if ( right_counter > 0 )
        colors.resize(colors.size()-right_counter);
    if ( left_counter > 0 or right_counter > 0 ) {
        std::vector<bool> new_endpoints(colors.size()*2);
        uint8_t j = 0;
        uint8_t left_ends = 0;
        uint8_t right_ends = 0;
        for ( uint8_t i = 0 ; j < new_endpoints.size() ; i++ ) {
            if ( endpoints[i] ) {
                if ( left_ends >= left_counter and left_ends < left_counter+colors.size() )
                    new_endpoints[j++] = true;
                left_ends++;
            } else {
                if ( right_ends >= left_counter and right_ends < left_counter+colors.size() )
                    new_endpoints[j++] = false;
                right_ends++;
            }
        }
        endpoints = new_endpoints;
    }
    left_barrier -= 2*left_counter;
    right_barrier -= 2*left_counter;
}
bool State::check() const {
    if ( colors.size() * 2 != endpoints.size() ) {
        std::cerr << "size mismatch " << std::endl;
        return false;
    }
    if ( left_barrier > right_barrier ) {
        std::cerr << "left barrier" << std::endl;
        return false;
    }
    if ( right_barrier > endpoints.size() ) {
        std::cerr << "right barrier" << std::endl;
        return false;
    }

    for ( uint8_t i = 0 ; i < left_barrier ; i++ ) {
        if ( endpoints[i] != true ) {
            std::cerr << "early end" << std::endl;
            return false;
        }
    }
    for ( uint8_t i = right_barrier ; i < endpoints.size() ; i++ ) {
        if ( endpoints[i] != false ) {
            std::cerr << "late start" << std::endl;
            return false;
        }
    }
    uint8_t counter = 0;
    for ( auto e : endpoints ) {
        if ( e ) {
            counter++;
        } else {
            if ( counter == 0 ) {
                std::cerr << "below zero" << std::endl;
                return false;
            }
            counter--;
        }
    }
    if ( counter != 0 ) {
        std::cerr << "unbalanced" << std::endl;
        return false;
    }
    return true;
}


uint8_t State::get_left_barrier() const {
    return left_barrier;
}
uint8_t State::get_right_barrier() const {
    return right_barrier;
}
uint8_t State::get_interval_count() const {
    return colors.size();
}
uint8_t State::get_interval_color(uint8_t i) const {
    return colors[i];
}
std::vector<bool> State::get_endpoints() const {
    return endpoints;
}
void State::add_interval(uint8_t left_endpoint, uint8_t right_endpoint, uint8_t color) {
    endpoints.resize(endpoints.size()+2);
    uint8_t left_count = 0;
    for ( unsigned i = endpoints.size() ; i > right_endpoint+2u ; ) { i--;
        endpoints[i] = endpoints[i-2];
    }
    for ( unsigned i = right_endpoint+1u ; i > left_endpoint+1u ; ) { i--;
        endpoints[i] = endpoints[i-1];
    }
    endpoints[left_endpoint] = true;
    endpoints[right_endpoint+1u] = false;
    for ( unsigned i = 0 ; i < left_endpoint ; i++ )
        if ( endpoints[i] )
            left_count++;
    colors.resize(colors.size()+1u);
    for ( uint8_t i = colors.size() ; i > left_count+1u ; ) { i--;
        colors[i] = colors[i-1];
    }
    colors[left_count] = color;
    right_barrier += 2;
    normalize_colors();
    check() or ( std::cerr << "FUCK (add) " << *this << std::endl );
}
void State::add_interval(const ColoredInterval cint) {
    add_interval(cint.left_endpoint, cint.right_endpoint, cint.color);
}

std::vector<Interval> State::get_intervals() const {
    std::vector<Interval> result(get_interval_count());
    uint8_t left_endpoints = 0;
    uint8_t right_endpoints = 0;
    for ( unsigned i = 0 ; i < endpoints.size() ; i++ ) {
        if ( endpoints[i] )
            result[left_endpoints++].left_endpoint = i;
        else
            result[right_endpoints++].right_endpoint = i;
    }
    return result;
}
std::vector<ColoredInterval> State::get_colored_intervals() const {
    auto intervals = get_intervals();
    std::vector<ColoredInterval> result(intervals.size());
    for ( unsigned i = 0 ; i < result.size() ; i++ )
        result[i] = { intervals[i].left_endpoint, intervals[i].right_endpoint, colors[i] };
    return result;
}
std::vector<uint8_t> State::get_clique_sizes() const {
    std::vector<uint8_t> cliques(endpoints.size()+1);
    uint8_t clique = 0;
    for ( uint8_t i = 0 ; i < endpoints.size() ; i++ ) {
        cliques[i] = clique;
        if ( endpoints[i] )
            clique++;
        else
            clique--;
    }
    cliques.back() = 0;
    return cliques;
}
uint8_t State::get_max_clique_size() const {
    auto clique_sizes = get_clique_sizes();
    return *std::max_element(clique_sizes.begin(), clique_sizes.end());
}
uint8_t State::get_color_count() const {
    if ( colors.size() )
        return *std::max_element(colors.begin(), colors.end()) + 1;
    return 0;
}
std::vector<uint8_t> State::get_started() const {
    std::vector<uint8_t> started(endpoints.size()+1);
    started[0]=0;
    for ( uint8_t i = 1 ; i <= endpoints.size() ; i++ )
        started[i] = started[i-1]+(endpoints[i-1]?1:0);
    return started;
}
std::vector<uint8_t> State::get_finished() const {
    std::vector<uint8_t> finished(endpoints.size()+1);
    finished[0]=0;
    for ( uint8_t i = 1 ; i <= endpoints.size() ; i++ )
        finished[i] = finished[i-1]+(endpoints[i-1]?0:1);
    return finished;
}
std::vector<std::pair<Interval,std::vector<uint8_t>>> State::get_possible_additions(uint8_t max_clique, uint8_t max_colors) const {
    std::vector<std::pair<Interval,std::vector<uint8_t>>> result;
    auto intervals = get_intervals();
    auto cliques = get_clique_sizes();
    auto used_colors = get_color_count();
    auto started = get_started();
    auto finished = get_finished();
    max_colors = std::min<uint8_t>(max_colors, used_colors+1);
    for ( uint8_t i = 0 ; i <= intervals.size() ; i++ ) {
        uint8_t left_a = 0;
        if ( i > 0 )
            left_a = intervals[i-1].left_endpoint+1;
        uint8_t left_b = endpoints.size();
        if ( i < intervals.size() )
            left_b = intervals[i].left_endpoint;
        uint8_t right_a = 0;
        if ( i > 0 )
            right_a = intervals[i-1].right_endpoint+1;
        uint8_t right_b = endpoints.size();
        if ( i < intervals.size() )
            right_b = intervals[i].right_endpoint;
        left_a = std::max(left_a, left_barrier);
        right_b = std::min(right_b, right_barrier);
        for ( uint8_t i = left_a ; i <= left_b ; i++ )
            for ( uint8_t j = std::max(i, right_a) ; j <= right_b ; j++ ) {
                bool ok = true;
                for ( uint8_t k = i ; k <= j ; k++ )
                    if ( cliques[k] >= max_clique ) {
                        ok = false;
                        break;
                    }
                if ( ok ) {
                    std::vector<bool> available_colors(max_colors, true);
                    for ( uint8_t k = finished[i] ; k < started[i] ; k++ )
                        available_colors[colors[k]] = false;
                    for ( uint8_t k = finished[j] ; k < started[j] ; k++ )
                        available_colors[colors[k]] = false;
                    std::vector<uint8_t> ac;
                    for ( uint8_t c = 0 ; c < max_colors ; c++ )
                        if ( available_colors[c] )
                            ac.emplace_back(c);
                    result.emplace_back(Interval(i, j), ac);
                }
            }
    }
    return result;
}
void State::reverse() {
    std::reverse(colors.begin(), colors.end());
    std::reverse(endpoints.begin(), endpoints.end());
    for ( uint8_t i = 0 ; i < endpoints.size() ; i++ )
        endpoints[i] = not endpoints[i];
    std::swap(left_barrier, right_barrier);
    left_barrier = endpoints.size()-left_barrier;
    right_barrier = endpoints.size()-right_barrier;
    normalize_colors();
    check() or ( std::cerr << "FUCK (rev) " << *this << std::endl );
}
void State::push_left_barrier() {
    while ( left_barrier < right_barrier and endpoints[left_barrier] == true )
        left_barrier++;
    if ( left_barrier < right_barrier ) {
        left_barrier++;
        drop_intervals();
        normalize_colors();
    }
    check() or ( std::cerr << "FUCK (left) " << *this << std::endl );
}
void State::push_right_barrier() {
    while ( left_barrier < right_barrier and endpoints[right_barrier-1] == false )
        right_barrier--;
    if ( left_barrier < right_barrier ) {
        right_barrier--;
        drop_intervals();
        normalize_colors();
    }
    check() or ( std::cerr << "FUCK (right) " << *this << std::endl );
}
StateCode State::encode() const {
    uint64_t interval_code = 0;
    uint64_t color_code = 0;
    uint8_t interval_count = get_interval_count();
    std::vector<uint8_t> options(colors.size());
    uint8_t next_color = 0;
    for ( uint8_t i = 0 ; i < interval_count ; i++ ) {
        options[i] = next_color + 1;
        next_color = std::max<uint8_t>(colors[i]+1u, next_color);
    }
    for ( uint8_t i = interval_count ; i > 0 ; ) { i--;
        color_code = color_code * options[i] + colors[i];
    }

    for ( uint8_t i = left_barrier ; i < right_barrier ; i++ )
        interval_code = interval_code*2 + (endpoints[i]?1:0);
    interval_code = interval_code*(right_barrier+1) + left_barrier;
    interval_code = interval_code*(2*interval_count+1) + right_barrier;
    interval_code = interval_code*256 + get_interval_count();
    return StateCode(interval_code, color_code);
}
void State::decode(StateCode state_code) {
    auto code = state_code.interval_code;
    uint8_t interval_count = code % 256 ; code /= 256;
    colors.resize(interval_count);
    endpoints.resize(2*interval_count);
    right_barrier = code % (2*interval_count+1) ; code /= 2*interval_count+1;
    left_barrier = code % (right_barrier+1) ; code /= right_barrier+1;
    for ( uint8_t i = 0 ; i < left_barrier ; i++ )
        endpoints[i] = true;
    for ( uint8_t i = right_barrier ; i < endpoints.size() ; i++ )
        endpoints[i] = false;
    for ( uint8_t i = right_barrier ; i > left_barrier ; ) { i--;
        endpoints[i] = code % 2 ; code /= 2;
    }
    code = state_code.color_code;
    uint8_t next_color = 0;
    for ( uint8_t i = 0 ; i < interval_count ; i++ ) {
        colors[i] = code % (next_color+1u) ; code /= (next_color+1u);
        next_color = std::max<uint8_t>(colors[i]+1u, next_color);
    }
}

std::string State::dump() const {
    std::string result(colors.size()*2 + 2, ' ');
    uint8_t left_endpoints = 0;
    uint8_t right_endpoints = 0;
    uint8_t shift = 0;
    for ( uint8_t i = 0 ; i < endpoints.size()+2 ; i++ ) {
        if ( i == left_barrier ) {
            result[i] = '[';
            shift++;
        }
        else if ( i == right_barrier+1 ) {
            result[i] = ']';
            shift++;
        }
        else if ( endpoints[i-shift] )
            result[i] = 'A' + colors[left_endpoints++];
        else
            result[i] = 'a' + colors[right_endpoints++];
    }
    return result;
}

void State::load(const std::string& code) {
    size_t shift = 0;
    uint8_t left_endpoints = 0;
    uint8_t right_endpoints = 0;
    assert( code.size() >= 2 and code.size()%2 == 0 );
    size_t size = (code.size() - 2) / 2;
    colors.resize(size);
    endpoints.resize(size*2);
    for ( size_t i = 0 ; i < code.size() ; i++ ) {
        if ( shift == 0 and code[i] == '[' ) {
            left_barrier = i;
            shift = 1;
        } else if ( shift == 1 and code[i] == ']' ) {
            right_barrier = i-1;
            shift = 2;
        } else if ( left_endpoints < size and shift < 2 and code[i] >= 'A' and code[i] <= 'Z' ) {
            endpoints[i-shift] = true;
            colors[left_endpoints++] = code[i] - 'A';
        } else if ( right_endpoints < size and shift > 0 and code[i] >= 'a' and code[i] <= 'z' ) {
            endpoints[i-shift] = false;
            assert( colors[right_endpoints++] == code[i] - 'a' );
        } else
            assert( false );
    }
    assert( left_endpoints == right_endpoints and right_endpoints == size );
    drop_intervals();
    normalize_colors();
}

void State::ostream(std::ostream& stream) const {
    stream << "<" << encode() << " " << dump() << " " << (unsigned)get_interval_count() << ", (" << (unsigned)get_left_barrier() << ", " << (unsigned)get_right_barrier() << " #";
    for ( auto e : get_endpoints() )
        stream << (e?1:0);
    stream << "#)";
    if ( get_interval_count() > 0 )
        stream << ":";
    for ( const auto& cint : get_colored_intervals() )
        stream << " " << cint;
    stream << ">";
    /*
    stream << " =>";
    for ( const auto& move_answer : get_available_moves(3,4) ) {
        auto move = move_answer.first;
        auto answer = move_answer.second;
        State t = *this;
        t.add_interval(move.left_endpoint, move.right_endpoint, 0);
        stream << " (" << (unsigned)move.left_endpoint << ", " << (unsigned)move.right_endpoint << ") @";
        for ( const auto& color : answer )
            stream << (unsigned)color;
        stream << (t.check()?" OK":" FAIL");
    }
    */
}
std::ostream& operator<< (std::ostream& stream, const State& value) {
    value.ostream(stream);
    return stream;
}
