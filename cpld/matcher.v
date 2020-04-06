// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module matcher(
    input clk,

    // RX path
    input rx_bit,
    input rx_bit_valid_now,
    input rx_byte_start,

    // TX path
    output reg tx_which_byte,
    output tx_trigger,
    input tx_done
);
    // 00 - matching
    // 01 - waiting for TX
    // 10 - succeeded
    // 11 - failed
    reg [1:0] matcher_state = 0;
    reg [5:0] matcher_bit_cnt = 0;

    // XXX this design is very very hardcoded for the specific bits to match
    // 16bit (4 digits) - "2891" - "1000 0010  0001 1001" (MSB first) - "0100 0001  1001 1000" (shift out order)
    // 64bit - "22681f5dd3196c1f" - "00100010 01101000 00011111 01011101 11010011 00011001 01101100 00011111" (MSB first)
    //                              - "01000100 00010110 11111000 10111010 11001011 10011000 00110110 11111000" (shift out order)

    // Repeating again, we want to match on the following patterns:
    // 0100 0001  1001 1000     (4 digit)
    // 0100 0100  0001 0110  1111 1000  1011 1010  1100 1011  1001 1000  0011 0110  1111 1000   (64bit)

    initial tx_which_byte <= 1;

    assign tx_trigger = matcher_state[1];

    wire legal_byte_start_state;

    assign legal_byte_start_state = matcher_bit_cnt == 0 ||
                                    matcher_bit_cnt == 8
`ifdef PATTERN_64
                                    ||
                                    matcher_bit_cnt == 16 ||
                                    matcher_bit_cnt == 24 ||
                                    matcher_bit_cnt == 32 ||
                                    matcher_bit_cnt == 40 ||
                                    matcher_bit_cnt == 48 ||
                                    matcher_bit_cnt == 56
`endif
                                    ;

    reg bit_is_expected;

    always @(*)
        case (matcher_bit_cnt)
`ifdef PATTERN_64
            0: bit_is_expected <= rx_bit == 0;      // Bit 0
            1: bit_is_expected <= rx_bit == 1;      // Bit 1
            2: bit_is_expected <= rx_bit == 0;      // Bit 2
            3: bit_is_expected <= rx_bit == 0;      // Bit 3
            4: bit_is_expected <= rx_bit == 0;      // Bit 4
            5: bit_is_expected <= rx_bit == 1;      // Bit 5
            6: bit_is_expected <= rx_bit == 0;      // Bit 6
            7: bit_is_expected <= rx_bit == 0;      // Bit 7

            8: bit_is_expected <= rx_bit == 0;      // Bit 8
            9: bit_is_expected <= rx_bit == 0;      // Bit 9
            10: bit_is_expected <= rx_bit == 0;      // Bit 10
            11: bit_is_expected <= rx_bit == 1;      // Bit 11
            12: bit_is_expected <= rx_bit == 0;      // Bit 12
            13: bit_is_expected <= rx_bit == 1;      // Bit 13
            14: bit_is_expected <= rx_bit == 1;      // Bit 14
            15: bit_is_expected <= rx_bit == 0;      // Bit 15

            16: bit_is_expected <= rx_bit == 1;      // Bit 16
            17: bit_is_expected <= rx_bit == 1;      // Bit 17
            18: bit_is_expected <= rx_bit == 1;      // Bit 18
            19: bit_is_expected <= rx_bit == 1;      // Bit 19
            20: bit_is_expected <= rx_bit == 1;      // Bit 20
            21: bit_is_expected <= rx_bit == 0;      // Bit 21
            22: bit_is_expected <= rx_bit == 0;      // Bit 22
            23: bit_is_expected <= rx_bit == 0;      // Bit 23

            24: bit_is_expected <= rx_bit == 1;      // Bit 24
            25: bit_is_expected <= rx_bit == 0;      // Bit 25
            26: bit_is_expected <= rx_bit == 1;      // Bit 26
            27: bit_is_expected <= rx_bit == 1;      // Bit 27
            28: bit_is_expected <= rx_bit == 1;      // Bit 28
            29: bit_is_expected <= rx_bit == 0;      // Bit 29
            30: bit_is_expected <= rx_bit == 1;      // Bit 30
            31: bit_is_expected <= rx_bit == 0;      // Bit 31

            32: bit_is_expected <= rx_bit == 1;      // Bit 32
            33: bit_is_expected <= rx_bit == 1;      // Bit 33
            34: bit_is_expected <= rx_bit == 0;      // Bit 34
            35: bit_is_expected <= rx_bit == 0;      // Bit 35
            36: bit_is_expected <= rx_bit == 1;      // Bit 36
            37: bit_is_expected <= rx_bit == 0;      // Bit 37
            38: bit_is_expected <= rx_bit == 1;      // Bit 38
            39: bit_is_expected <= rx_bit == 1;      // Bit 39

            40: bit_is_expected <= rx_bit == 1;      // Bit 40
            41: bit_is_expected <= rx_bit == 0;      // Bit 41
            42: bit_is_expected <= rx_bit == 0;      // Bit 42
            43: bit_is_expected <= rx_bit == 1;      // Bit 43
            44: bit_is_expected <= rx_bit == 1;      // Bit 44
            45: bit_is_expected <= rx_bit == 0;      // Bit 45
            46: bit_is_expected <= rx_bit == 0;      // Bit 46
            47: bit_is_expected <= rx_bit == 0;      // Bit 47

            48: bit_is_expected <= rx_bit == 0;      // Bit 48
            49: bit_is_expected <= rx_bit == 0;      // Bit 49
            50: bit_is_expected <= rx_bit == 1;      // Bit 50
            51: bit_is_expected <= rx_bit == 1;      // Bit 51
            52: bit_is_expected <= rx_bit == 0;      // Bit 52
            53: bit_is_expected <= rx_bit == 1;      // Bit 53
            54: bit_is_expected <= rx_bit == 1;      // Bit 54
            55: bit_is_expected <= rx_bit == 0;      // Bit 55

            56: bit_is_expected <= rx_bit == 1;      // Bit 56
            57: bit_is_expected <= rx_bit == 1;      // Bit 57
            58: bit_is_expected <= rx_bit == 1;      // Bit 58
            59: bit_is_expected <= rx_bit == 1;      // Bit 59
            60: bit_is_expected <= rx_bit == 1;      // Bit 60
            61: bit_is_expected <= rx_bit == 0;      // Bit 61
            62: bit_is_expected <= rx_bit == 0;      // Bit 62
            default: bit_is_expected <= rx_bit == 0; // Bit 63
`else
            0: bit_is_expected <= rx_bit == 0;      // Bit 0
            1: bit_is_expected <= rx_bit == 1;      // Bit 1
            2: bit_is_expected <= rx_bit == 0;      // Bit 2
            3: bit_is_expected <= rx_bit == 0;      // Bit 3
            4: bit_is_expected <= rx_bit == 0;      // Bit 4
            5: bit_is_expected <= rx_bit == 0;      // Bit 5
            6: bit_is_expected <= rx_bit == 0;      // Bit 6
            7: bit_is_expected <= rx_bit == 1;      // Bit 7

            8: bit_is_expected <= rx_bit == 1;      // Bit 8
            9: bit_is_expected <= rx_bit == 0;      // Bit 9
            10: bit_is_expected <= rx_bit == 0;      // Bit 10
            11: bit_is_expected <= rx_bit == 1;      // Bit 11
            12: bit_is_expected <= rx_bit == 1;      // Bit 12
            13: bit_is_expected <= rx_bit == 0;      // Bit 13
            14: bit_is_expected <= rx_bit == 0;      // Bit 14
            default: bit_is_expected <= rx_bit == 0; // Bit 15
`endif
        endcase

    always @(posedge clk) begin
        casex (matcher_state)
            2'b00: begin
                if (rx_bit_valid_now) begin
                    if (rx_byte_start && !legal_byte_start_state) begin
                        // Somehow desync? Don't report failure but restart
                        if (rx_bit != 0)            // Bit 0 _duplicated_
                            tx_which_byte <= 0;
                        matcher_bit_cnt <= 1;
                    end else
                        if (!bit_is_expected)
                            tx_which_byte <= 0;
`ifdef PATTERN_64
                        if (matcher_bit_cnt == 63)
`else
                        if (matcher_bit_cnt == 15)
`endif
                            matcher_state <= 2'b10;
                        matcher_bit_cnt <= matcher_bit_cnt + 1;
                end
            end

            // We know we cannot finish in one cycle; don't check here
            2'b1?: matcher_state <= 2'b01;

            2'b01:
                if (tx_done) begin
                    matcher_state <= 2'b00;
                    matcher_bit_cnt <= 0;
                    tx_which_byte <= 1;
                end
        endcase 
    end
endmodule
