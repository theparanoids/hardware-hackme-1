// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

module top(clk_, uart_tx, uart_rx, led0, led1);
    (* LOC = "FB2_5" *)
    input clk_;
    wire clk;
    BUFG bufg0 (.I(clk_), .O(clk));

    (* LOC = "FB2_2" *)
    output uart_tx;
    (* LOC = "FB2_1" *)
    input uart_rx;

    (* LOC = "FB1_1" *)
    output led0;
    (* LOC = "FB1_2" *)
    output led1;

    wire tx_which_byte;
    wire tx_trigger;
    wire tx_done;
    wire rx_bit;
    wire rx_bit_valid_now;
    wire rx_byte_start;

    assign led0 = rx_bit_valid_now;
    assign led1 = tx_trigger;

    txpath txpath(
        .clk_8mhz(clk),
        .which_byte(tx_which_byte),
        .trigger(tx_trigger),
        .tx_wire(uart_tx),
        .done(tx_done)
    );

    rxpath rxpath(
        .clk_8mhz(clk),
        .rx_wire(uart_rx),
        .out_bit(rx_bit),
        .valid_now(rx_bit_valid_now),
        .byte_start(rx_byte_start)
    );

    matcher matcher(
        .clk(clk),

        .rx_bit(rx_bit),
        .rx_bit_valid_now(rx_bit_valid_now),
        .rx_byte_start(rx_byte_start),

        .tx_which_byte(tx_which_byte),
        .tx_trigger(tx_trigger),
        .tx_done(tx_done)
    );
endmodule
