# Hướng Dẫn Cân Màu Màn Hình Kép (Dual Monitor Calibration Guide)

Tài liệu ghi nhận quy trình cân chỉnh màu sắc phần cứng (DDC/CI) cho hệ thống 2 màn hình **Acer AL258Q** (Panel Fast IPS 320Hz, Controller CVTE, Firmware 46507).

---

## 1. Thông Số Cấu Hình Cân Màu Chuẩn (Đã Lưu)

| Thiết lập phần cứng | Màn hình Trái (Monitor 0) | Màn hình Phải (Monitor 1) | Ghi chú kỹ thuật |
| :--- | :---: | :---: | :--- |
| **Cổng kết nối** | DisplayPort 1 (`0x60 = 15`) | DisplayPort 2 (`0x60 = 16`) | Nhận diện qua Windows Display 1 & Display 2 |
| **Độ sáng (Luminance - 0x10)** | **70** | **70** | ~230–250 nits, tối ưu cho phòng có ánh sáng đèn |
| **Độ tương phản (Contrast - 0x12)** | **50** | **50** | Mức chuẩn bảo toàn dải 0–255, không bệt highlight |
| **Chế độ màu (Color Preset - 0x14)** | **5** (6500K D65) | **11** (User Color Mode) | Màn phải cần Preset 11 để mở khóa phần cứng DAC |
| **Red Gain (0x16)** | **50** | **47** | Hạ 3 nấc để triệt tiêu hoàn toàn hiện tượng ửng đỏ |
| **Green Gain (0x18)** | **50** | **49** | Hạ 1 nấc để chống ngả vàng khi dải đỏ giảm |
| **Blue Gain (0x1A)** | **50** | **51** | Tăng 1 nấc để nền trắng trong trẻo, khớp với màn trái |
| **Tần số quét (Vertical Freq - 0xAE)** | 320.00 Hz | 320.00 Hz | Tần số quét gốc của tấm nền AL258Q |

---

## 2. Phát Hiện Kỹ Thuật Quan Trọng (CVTE Scaler Lock)

- Trên dòng màn hình Acer trang bị vi điều khiển **CVTE**, khi màn hình đặt ở `0x14 = 5` (Preset 6500K mặc định), vi điều khiển sẽ **khóa bảng màu phần cứng (Hardware LUT/DAC Lock)**. Các thay đổi đối với `0x16 (Red)`, `0x18 (Green)`, `0x1A (Blue)` sẽ không có hiệu lực thị giác trực tiếp.
- **Giải pháp**: Phải chuyển `0x14` sang **`11` (User 1 / Custom)** trên màn hình cần bù trừ màu. Khi đó phần cứng DAC mở khóa hoàn toàn, các lệnh vi sai RGB Gain sẽ tác động tức thì vào dòng điện cấp cho từng sub-pixel.

---

## 3. Danh Sách File Profile Đã Tạo

Các profile được lưu dưới định dạng JSON trong thư mục `profiles/`:

1. **`profiles/Calibrated-Dual-Setup.json`**:
   - Chứa toàn bộ cấu hình độc lập cho cả 2 màn hình (Trái & Phải).
   - Được áp dụng đồng thời qua lệnh `apply-dual`.
   - Mặc định **bỏ qua cổng vào (0x60)** để tránh màn hình đen khi thứ tự cổng thay đổi. Chỉ thêm `--include-input` khi bạn chắc chắn cách đấu dây.
2. **`profiles/Calibrated-Left.json`**:
   - Profile riêng cho Màn hình Trái (DP1 - 20 thanh ghi).
3. **`profiles/Calibrated-Right.json`**:
   - Profile riêng cho Màn hình Phải (DP2 - 32 thanh ghi).
4. **`profiles/Color-Accurate-D65.json`**:
   - Profile chuẩn 6500K sRGB tổng quát.

---

## 4. Cách Sử Dụng Lệnh CLI (lịch sử)

> Lịch sử: CLI Python `sysvane-ddc` đã bị xoá (còn trong commit `30b7852`). Các lệnh dưới đây không chạy được cho tới khi module DDC bằng Rust có lệnh tương đương.

### Áp dụng cho cả 2 màn hình cùng lúc (Khuyên dùng)
```powershell
uv run sysvane-ddc apply-dual
```

### Áp dụng theo từng màn hình
```powershell
# Áp dụng cho Màn hình Trái (Index 0):
uv run sysvane-ddc apply-profile Calibrated-Left --monitor 0

# Áp dụng cho Màn hình Phải (Index 1):
uv run sysvane-ddc apply-profile Calibrated-Right --monitor 1
```

### Xem danh sách tất cả các profile hiện có
```powershell
uv run sysvane-ddc list-profiles
```
