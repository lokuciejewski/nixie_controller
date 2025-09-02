import serial
import time
import datetime


def get_datetime(ser: serial.Serial) -> int:
    command = [0x69, 0x1, 0x0, 0x5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ser.write(command)
    resp = ser.read(16)
    resp = [int(byte) for byte in resp]
    print(resp)
    print(resp[4:6])
    year = int.from_bytes(resp[4:6], byteorder="little")
    month = resp[6]
    day = resp[7]
    hour = resp[8]
    minute = resp[9]
    second = resp[10]
    return time.mktime((year, month, day, hour, minute, second, 0, 0, 0))


def set_datetime(ser: serial.Serial, datetime_struct):
    command = [
        0x69,
        0x1,
        0x0,
        0x2,
        datetime_struct.year & 0xFF,
        (datetime_struct.year >> 8) & 0xFF,
        datetime_struct.month,
        datetime_struct.day,
        datetime_struct.hour,
        datetime_struct.minute,
        datetime_struct.second,
        0,
        0,
        0,
        0,
        0,
    ]
    ser.write(command)


def main():
    with serial.Serial("/dev/ttyUSB0", 115200) as ser:
        current_time = get_datetime(ser)
        print(time.localtime(current_time))
        time.sleep(0.5)
        # set_datetime(ser, datetime.datetime.now())
        # time.sleep(0.5)
        # current_time = get_datetime(ser)
        # print(time.localtime(current_time))


if __name__ == "__main__":
    main()
