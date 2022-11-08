 import micropython
import select
import sys
import machine
import array
import time
import _thread

NUM_LEDS = 60
PIN_NUM = 7
brightness = 0.9

@rp2.asm_pio(sideset_init=rp2.PIO.OUT_LOW, out_shiftdir=rp2.PIO.SHIFT_LEFT, autopull=True, pull_thresh=24)
def ws2812():
    T1 = 2
    T2 = 5
    T3 = 3
    wrap_target()
    label("bitloop")
    out(x, 1)               .side(0)    [T3 - 1]
    jmp(not_x, "do_zero")   .side(1)    [T1 - 1]
    jmp("bitloop")          .side(1)    [T2 - 1]
    label("do_zero")
    nop()                   .side(0)    [T2 - 1]
    wrap()

# Create the StateMachine with the ws2812 program, outputting on pin
sm = rp2.StateMachine(0, ws2812, freq=8_000_000, sideset_base=machine.Pin(PIN_NUM))

# Start the StateMachine, it will wait for data on its FIFO.
sm.active(1)

arr = [0x000F0F0F for _ in range(NUM_LEDS)]

def pixels_show():
    dimmer_ar = array.array("I", [0 for _ in range(NUM_LEDS)])
    for i,c in enumerate(arr):
        r = int(((c >> 8) & 0xFF) * brightness)
        g = int(((c >> 16) & 0xFF) * brightness)
        b = int((c & 0xFF) * brightness)
        dimmer_ar[i] = (g<<16) + (r<<8) + b
    sm.put(dimmer_ar, 8)
    time.sleep_ms(10)

cur_buf = ""

def buffer_to_arr():
    colors = cur_buf.split(";")
    for i in range(0, min(len(colors), NUM_LEDS)):
        rgb = colors[i].split(",")
        r = rgb[0]
        g = rgb[1]
        b = rgb[2]
        # print(r)
        # print(g)
        # print(b)
        arr[i] = (int(r) << 8) + (int(g) << 16) + int(b)
        
    print(arr)

micropython.kbd_intr(-1)
pixels_show()

#def button_watch():
#    button = machine.Pin(12, machine.Pin.IN, machine.Pin.PULL_UP)
#    while True:
#        if button.value() == 1:
#            print("BUTTON_DOWN");
#        else:
#            print("BUTTON_UP");
#        time.sleep_ms(250)
#        
#_thread.start_new_thread(button_watch, ())

while True:
  while sys.stdin in select.select([sys.stdin], [], [], 0)[0]:        
    ch = sys.stdin.read(1)
    if ch == '\n':
        print("end line")
        buffer_to_arr()
        cur_buf = ""
        pixels_show()
    else:
        cur_buf += ch
