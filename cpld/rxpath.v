// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module rxpath(
    input clk_8mhz,
    input rx_wire,
    output out_bit,
    output valid_now,
    output byte_start
);

    // Known bug: Immediate back-to-back bytes don't work; need a small delay

    // Synchronizer
    reg rx_wire_clk1 = 0;
    reg rx_wire_clk2 = 0;
    always @(posedge clk_8mhz) begin
        rx_wire_clk1 <= rx_wire;
        rx_wire_clk2 <= rx_wire_clk1;
    end
    assign out_bit = rx_wire_clk2;

    // Falling edge detect
    reg rx_wire_clk3 = 0;
    always @(posedge clk_8mhz)
        rx_wire_clk3 <= rx_wire_clk2;

    // Used to divide to get 1 MHz
    reg [2:0] cyc_counter = 0;
    // Used to remember which bit we should be outputting
    reg [3:0] bit_counter = 0;

    wire falling_edge_detected;
    assign falling_edge_detected =
    	bit_counter == 0 && rx_wire_clk3 == 1 && rx_wire_clk2 == 0;

    always @(posedge clk_8mhz) begin
        if (falling_edge_detected == 1 && bit_counter == 0)
            cyc_counter <= 0;
        else
            cyc_counter <= cyc_counter + 1;
    end

    always @(posedge clk_8mhz) begin
        if (falling_edge_detected == 1 && bit_counter == 0)
            bit_counter <= 1;
        else if (cyc_counter == 7)
            if (bit_counter == 10)
                bit_counter <= 0;
            else if (bit_counter != 0)
                bit_counter <= bit_counter + 1;
    end

    assign byte_start = bit_counter == 2;
    assign valid_now = cyc_counter == 2 && (bit_counter >= 2 && bit_counter <= 9);

endmodule
