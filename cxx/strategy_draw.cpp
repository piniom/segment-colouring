#include <cassert>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stack>
#include <unordered_map>

#include <boost/program_options.hpp>
namespace bpo = boost::program_options;

#include "state.hpp"

std::unordered_map<StateCode, size_t> known_codes;
std::vector<StateCode> codes;
std::vector<Interval> moves;
std::vector<std::vector<std::pair<uint8_t, size_t>>> responses;


std::string tikz_color(unsigned index) {
    static std::vector<std::string> map = {
        "red", "green", "blue",
        "cyan", "magenta", "yellow",
        "maroon", "lime", "navy"
        "purple", "olive", "teal",
        "fuchsia" };
    return map[index%map.size()];
};

std::string state_tikz_empty(const State& state, const Interval& move, const std::vector<std::pair<uint8_t, size_t>>& responses) {
    std::stringstream result;
    result << "\\begin{tikzpicture}\n";
    int color_count = state.get_color_count();
    int endpoint_count = 2*state.get_interval_count();
    result << "\\draw [rounded corners,black,opacity=0,fill] (-1,-2) rectangle (" << endpoint_count << "," << color_count << ");\n";
    result << "\\end{tikzpicture}";
    return result.str();
}
std::string state_tikz(const State& state, const Interval& move, const std::vector<std::pair<uint8_t, size_t>>& responses) {
    std::stringstream result;
    result << "\\begin{tikzpicture}\n";
    int color_count = state.get_color_count();
    int endpoint_count = 2*state.get_interval_count();
    result << "\\draw [rounded corners,black,opacity=0.1,fill] (-1,-2) rectangle (" << endpoint_count << "," << color_count << ") node [black,opacity=0.3,above,anchor=north east] {" << state.dump() << "};\n";
    if ( true ) {
        float left_barrier = state.get_left_barrier();
        float right_barrier = state.get_right_barrier();
        left_barrier -= 0.85;
        right_barrier -= 0.15;
        result << "\\draw [dashed,black,opacity=0.9] (" << left_barrier << ",-0.5) -- (" << left_barrier << "," << color_count-0.5 << ");\n";
        result << "\\draw [dashed,black,opacity=0.9] (" << right_barrier << ",-0.5) -- (" << right_barrier << "," << color_count-0.5 << ");\n";
        float last_left = 0;
        float first_right = endpoint_count;
        for ( const auto& cint : state.get_colored_intervals() ) {
            int color = cint.color;
            float left = cint.left_endpoint;
            float right = cint.right_endpoint;
            last_left = std::max(last_left, left);
            first_right = std::min(first_right, right);
            result << "\\draw [|-|,thick," << tikz_color(color) << ",opacity=0.9] (" << left << "," << color << ") -- (" << right << "," << color << ") node [pos=0.5,anchor=south] {" << char('A'+color) << "};\n";
        }
        if ( move == Interval(0, 255) ) {
            float left = last_left-0.5;
            float right = right_barrier;
            result << "\\draw [<-,thick,black,opacity=0.9] (" << left << ",-1) -- (" << right << ",-1) node [pos=0.5,anchor=south] {};\n";
            result << "\\draw [dashed,black,opacity=0.9] (" << left << ",-1) -- (" << left << ",-0.5);\n";
        } else if ( move == Interval(1, 255) ) {
            float left = left_barrier;
            float right = first_right+0.5;
            result << "\\draw [->,thick,black,opacity=0.9] (" << left << ",-1) -- (" << right << ",-1) node [pos=0.5,anchor=south] {};\n";
            result << "\\draw [dashed,black,opacity=0.9] (" << right << ",-1) -- (" << right << ",-0.5);\n";
        } else {
            float left = move.left_endpoint;
            float right = move.right_endpoint;
            left -= 0.7;
            right -= 0.3;
            auto symbol = (responses.size()?"?":"\\textbf{X}");
            result << "\\draw [|-|,thick,black,opacity=0.9] (" << left << ",-1) -- (" << right << ",-1) node [pos=0.5,anchor=south] {" << symbol << "};\n";
        }
    }
    result << "\\end{tikzpicture}";
    return result.str();
}

int main(int argc, char** argv) {
    bool debug(false);
    std::vector<std::string> input_files(1, "-");

    std::ios_base::sync_with_stdio(false);
    std::string app_name = std::filesystem::path(argv[0]).stem().string();
    bpo::options_description desc("Options");
    desc.add_options()
        ("help", "Print help messages")
        ("debug", "Print debug information")
        ("input-file", bpo::value<std::vector<std::string>>(&input_files), "Input file")
        ;
    bpo::positional_options_description pos;
    pos.add("input-file", -1);
    bpo::variables_map var_map;
    try {
        bpo::store(bpo::command_line_parser(argc, argv).options(desc).positional(pos).run(), var_map);
        bpo::notify(var_map);
        if ( var_map.count("help") ) {
            std::cerr << app_name << std::endl;
            std::cerr << desc << std::endl;
            return 0;
        }
        if ( var_map.count("debug") )
            debug = true;
    } catch ( bpo::error& e ) {
        std::cerr << "ERROR: " << e.what() << std::endl << std::endl;
        std::cerr << desc << std::endl;
        return 1;
    }

    size_t max_clique = 0;
    size_t max_colors = 0;
    size_t max_size = 0;
    size_t max_depth = 0;
    for ( const auto& src : input_files ) {
        std::istream* stream = &std::cin;
        if ( src != "-" ) {
            stream = new std::ifstream(src.c_str());
            if ( not stream->good() ) {
                std::cerr << "Failed to load file '" << src << "'." << std::endl;
                return 1;
            }
        }
        size_t new_max_clique;
        size_t new_max_colors;
        for ( std::string line ; std::getline(*stream, line) ; ) {
            auto first = line.find_first_not_of(" \t\r\n");
            if ( first != std::string::npos and line[first] != '#' and line[first] != ';' ) {
                std::istringstream linestream(line);
                linestream >> new_max_clique >> new_max_colors;
                new_max_colors--;
                break;
            }
        }
        assert( new_max_clique >= 1 and new_max_clique <= 10 );
        assert( new_max_colors >= 1 and new_max_colors <= 20 );
        if ( (max_clique != 0 and new_max_clique != max_clique) or (max_colors != 0 and new_max_colors != max_colors ) ) {
            std::cerr << "Different input files play in different games." << std::endl;
            return 1;
        }
        max_clique = new_max_clique;
        max_colors = new_max_colors;
        for ( std::string line ; std::getline(*stream, line) ; ) {
            auto first = line.find_first_not_of(" \t\r\n");
            if ( first != std::string::npos and line[first] != '#' and line[first] != ';' ) {
                std::istringstream linestream(line);
                std::string state_to_load;
                linestream >> state_to_load;
                State state;
                state.load(state_to_load);
                if ( state.dump() != state_to_load ) {
                    std::cerr << "Strange state " << state_to_load << " (!=" << state.dump() << ")." << std::endl;
                    return 1;
                }
                auto code = state.encode();
                State rev_state = state;
                rev_state.reverse();
                auto rev_code = rev_state.encode();
                max_size = std::max<size_t>(max_size, state.get_interval_count());
                Interval interval;
                Interval rev_interval;
                std::string state_move;
                linestream >> state_move;
                if ( state_move == "<" ) {
                    // PUSH RIGHT BARRIER
                    interval = Interval(0, 255);
                    rev_interval = Interval(1, 255);
                } else if ( state_move == ">" ) {
                    // PUSH LEFT BARRIER
                    interval = Interval(1, 255);
                    rev_interval = Interval(0, 255);
                } else {
                    // INTRODUCE INTERVAL
                    size_t move_left = std::stoi(state_move);
                    linestream >> state_move;
                    size_t move_right = std::stoi(state_move);
                    interval = Interval(move_left, move_right);
                    rev_interval = Interval(move_right, move_left);
                }
                if ( known_codes.find(code) != known_codes.end() ) {
                    if ( codes[known_codes[code]] == code and moves[known_codes[code]] == interval )
                        continue;
                    if ( codes[known_codes[code]] == rev_code and moves[known_codes[code]] == rev_interval )
                        continue;
                    std::cerr << "Multiple conflicting definitions for " << state.dump() << ", " << rev_state.dump() << "." << std::endl;
                    return 1;
                }
                known_codes[code] = codes.size();
                known_codes[rev_code] = codes.size();
                codes.emplace_back(code);
                moves.emplace_back(interval);
            }
        }
        if ( src != "-" ) {
            delete stream;
        }
    }
    responses.resize(codes.size());
    for ( size_t i = 0 ; i < codes.size() ; i++ ) {
        State state;
        state.decode(codes[i]);
        auto move = moves[i];
        if ( move == Interval(0, 255) or move == Interval(1, 255) ) {
            if ( state.get_left_barrier() >= state.get_right_barrier() ) {
                std::cerr << "Cannot push barrier in " << state.dump() << "." << std::endl;
                return 1;
            }
            State result = state;
            if ( move == Interval(0, 255) )
                result.push_right_barrier();
            else
                result.push_left_barrier();
            auto code = result.encode();
            if ( known_codes.find(code) == known_codes.end() ) {
                std::cerr << "Answer for ( " << state.dump() << (move == Interval(0, 255)?" < ":" > ") << " ): " << result.dump() << " not found in the strategy." << std::endl;
                return 1;
            }
            responses[i].emplace_back(255,known_codes[code]);
        } else {
            bool found = false;
            for ( const auto& move_responses : state.get_possible_additions(max_clique, max_colors) ) {
                if ( move_responses.first == move ) {
                    found = true;
                    for ( auto& response : move_responses.second ) {
                        ColoredInterval cint(move, response);
                        State result = state;
                        result.add_interval(cint);
                        auto code = result.encode();
                        if ( known_codes.find(code) == known_codes.end() ) {
                            std::cerr << "Answer for ( " << state.dump() << ", " << move << ", " << char('A'+response) << " ): " << result.dump() << " not found in the strategy." << std::endl;
                            return 1;
                        }
                        responses[i].emplace_back(response, known_codes[code]);
                    }
                }
            }
            if ( not found ) {
                std::cerr << "Move " << move << " for " << state.dump() << " is not valid." << std::endl;
                return 1;
            }

        }
        if ( debug ) {
            std::cerr << " State #" << i << " is " << state.dump() << " with move ";
            if ( move == Interval(0, 255) )
                std::cerr << "push one right";
            else if ( move == Interval(1, 255) )
                std::cerr << "push one left";
            else
                std::cerr << move;
            std::cerr << " and ";
            if ( responses[i].size() == 0 )
                std::cerr << "no possible responses";
            else {
                std::cerr << "possible responses {";
                for ( const auto& response : responses[i] ) {
                    std::cerr << " " << response.second;
                }
                std::cerr << " }";
            }
            std::cerr << std::endl;
        }
    }
    std::vector<std::vector<size_t>> inverse(codes.size());
    std::vector<size_t> in_counter(codes.size(), 0);
    std::vector<size_t> depth(codes.size(), 0);
    std::stack<size_t> wins;
    size_t win_count = 0;
    for ( size_t i = 0 ; i < codes.size() ; i++ ) {
        if ( responses[i].size() == 0 ) {
            win_count++;
            wins.push(i);
            depth[i] = 1;
        }
        for ( auto response : responses[i] ) {
            inverse[response.second].emplace_back(i);
        }
    }
    while ( wins.size() > 0 ) {
        size_t win = wins.top(); wins.pop();
        for ( size_t other : inverse[win] ) {
            in_counter[other]++;
            depth[other] = std::max(depth[other], depth[win]+1);
            if ( in_counter[other] == responses[other].size() ) {
                win_count++;
                wins.push(other);
            }
        }
    }
    if ( win_count == codes.size() ) {
        std::cerr << "Strategy is correct." << std::endl;
        for ( size_t i = 0 ; i < codes.size() ; i++ ) {
            max_depth = std::max(max_depth, depth[i]);
        }
        std::cerr << "Maximum strategy depth is " << max_depth << "." << std::endl;
    } else {
        std::cerr << "Strategy is not correct. " << win_count << "/" << codes.size() << " states are wins." << std::endl;
        return 1;
    }
    State state;
    if ( known_codes.find(state.encode()) == known_codes.end() )
        std::cerr << "Base state is not included in the strategy." << std::endl;

    std::cout << "\\documentclass[preview]{standalone}\n";
    std::cout << "\\usepackage{tikz}\n";
    std::cout << "\\usetikzlibrary{graphs, quotes, graphdrawing}\n";
    std::cout << "\\usegdlibrary{layered}\n";
    std::cout << "\\begin{document}\n";
    std::cout << "\\begin{tikzpicture}\n";
    std::cout << "\\graph [layered layout] {\n";
    for ( size_t i = 0 ; i < codes.size() ; i++ ) {
        state.decode(codes[i]);
        std::cout << "node" << i << " [as={" << state_tikz_empty(state, moves[i], responses[i]) << "},label={center:" << state_tikz(state, moves[i], responses[i]) << "}];\n" << std::endl;
    }
    for ( size_t i = 0 ; i < codes.size() ; i++ ) {
        for ( const auto& color_result : responses[i] ) {
            auto color = color_result.first;
            auto result = color_result.second;
            if ( color != 255 )
                std::cout << "(node" << i << ") --[" << tikz_color(color) << ",\"" << char('A'+color) << "\"] (node" << result << ");\n";
            else
                std::cout << "(node" << i << ") -- (node" << result << ");\n";
        }
    }
    std::cout << "};\n";
    std::cout << "\\end{tikzpicture}\n";
    std::cout << "\\end{document}\n";
    return 0;
}
