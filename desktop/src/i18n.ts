import { useSyncExternalStore } from "react";

export type Lang = "auto" | "en" | "vi";

/** English is the source text and the key. `t` only takes a string listed here, so a missing translation fails the build. */
const VI = {
  Settings: "Cài đặt",
  About: "Giới thiệu",
  Version: "Phiên bản",
  Updates: "Cập nhật",
  "You have the latest version.": "Bạn đang dùng bản mới nhất.",
  "Checking…": "Đang kiểm tra…",
  "Check for updates": "Kiểm tra cập nhật",
  Appearance: "Giao diện",
  Language: "Ngôn ngữ",
  Theme: "Chủ đề",
  "Background program": "Chương trình nền",
  "Start with Windows is in the tray menu, because the background program owns it.": "Tùy chọn khởi động cùng Windows nằm trong menu khay, vì chương trình nền quản lý nó.",
  "Settings and logs are kept in the SysVane folder under ProgramData.": "Cài đặt và nhật ký nằm trong thư mục SysVane ở ProgramData.",
  Licences: "Giấy phép",
  "CPU temperature uses PawnIO by namazso (driver GPL-2.0, modules LGPL-2.1), downloaded from its official release when SysVane is installed.": "Nhiệt độ CPU dùng PawnIO của namazso (driver GPL-2.0, module LGPL-2.1), tải từ bản phát hành chính thức khi cài SysVane.",
  Overview: "Tổng quan",
  "SysVane {v} is available.": "Đã có SysVane {v}.",
  "Install and restart": "Cài và khởi động lại",
  "Installing…": "Đang cài…",
  "Update {v}": "Cập nhật {v}",
  "No changelog.": "Không có ghi chú thay đổi.",
  Later: "Để sau",
  Performance: "Hiệu năng",
  Processes: "Tiến trình",
  Pages: "Trang",
  "Core connected": "Core đang kết nối",
  "Core disconnected": "Core mất kết nối",
  "Theme: {mode}": "Giao diện: {mode}",
  "Theme: auto follows Windows": "Giao diện: tự động theo Windows",
  "Language: {mode}": "Ngôn ngữ: {mode}",
  auto: "tự động",
  light: "sáng",
  dark: "tối",
  English: "English",
  Vietnamese: "Tiếng Việt",
  "sysvane-core is not running": "sysvane-core chưa chạy",
  "SysVane reads sensors in a separate background program so this window stays light. It normally starts by itself and shows in the tray.": "SysVane đọc cảm biến trong một chương trình nền riêng để cửa sổ này luôn nhẹ. Thường nó tự khởi động và nằm ở khay hệ thống.",
  "Start sysvane-core": "Khởi động sysvane-core",
  "Started. This window fills in within a few seconds. If nothing changes, sysvane-core.exe is missing from the folder next to sysvane-ui.exe.": "Đã khởi động. Cửa sổ này sẽ có số liệu sau vài giây. Nếu không đổi, thiếu sysvane-core.exe trong thư mục chứa sysvane-ui.exe.",
  "Reading the first sensor values…": "Đang đọc giá trị cảm biến đầu tiên…",
  "The core stopped answering. Showing the last reading; reconnecting.": "Core ngừng phản hồi. Đang hiện số liệu cuối cùng và thử kết nối lại.",
  "n/a": "không có",
  none: "không có",
  Yes: "Có",
  No: "Không",
  "{n} minutes ago": "{n} phút trước",
  now: "bây giờ",
  Down: "Tải xuống",
  Up: "Tải lên",
  Receive: "Nhận",
  Send: "Gửi",
  Memory: "Bộ nhớ",
  Network: "Mạng",
  Temperature: "Nhiệt độ",
  Utilization: "Mức sử dụng",
  Threads: "Luồng",
  Cached: "Cache",
  Free: "Trống",
  Used: "Đã dùng",
  Type: "Loại",
  Total: "Tổng",
  "{a} of {b}": "{a} / {b}",
  "CPU temperature needs administrator rights. The installed SysVane starts with them.": "Nhiệt độ CPU cần quyền quản trị. SysVane đã cài sẽ tự chạy với quyền này.",
  Resource: "Tài nguyên",
  Usage: "Mức dùng",
  Now: "Hiện tại",
  Detail: "Chi tiết",
  "Last {n} minutes": "{n} phút gần nhất",
  "Disk {d}": "Ổ đĩa {d}",
  "clock unavailable": "không có xung nhịp",
  "{used} of {total}, {cached} cached": "{used} / {total}, {cached} cache",
  "{v} VRAM": "{v} VRAM",
  "read {r}, write {w}": "đọc {r}, ghi {w}",
  Temperatures: "Nhiệt độ",
  "red line at {t} °C": "đường đỏ ở {t} °C",
  "No sensor is readable.": "Không đọc được cảm biến nào.",
  "Top CPU": "CPU cao nhất",
  "Top memory": "Bộ nhớ cao nhất",
  System: "Hệ thống",
  Processor: "Bộ xử lý",
  "Cores / threads": "Nhân / luồng",
  Graphics: "Đồ họa",
  "Graphics driver": "Driver đồ họa",
  Handles: "Handle",
  "{host} · {os} (build {build}) · up {up}": "{host} · {os} (bản dựng {build}) · hoạt động {up}",
  Resources: "Tài nguyên",
  Speed: "Tốc độ",
  Overall: "Tổng thể",
  "Logical processors": "Bộ xử lý logic",
  "CPU view": "Chế độ xem CPU",
  "Clock speed": "Xung nhịp",
  "clock n/a": "không có xung",
  "Temperature (Tctl)": "Nhiệt độ (Tctl)",
  "Base speed": "Xung cơ bản",
  Cores: "Nhân",
  "Up time": "Thời gian hoạt động",
  Vendor: "Hãng",
  Architecture: "Kiến trúc",
  "In use": "Đang dùng",
  Available: "Khả dụng",
  Committed: "Đã cam kết",
  "Memory in use": "Bộ nhớ đang dùng",
  "Paged pool": "Paged pool",
  "Non-paged pool": "Non-paged pool",
  Swap: "Swap",
  "Dedicated memory": "Bộ nhớ riêng",
  "Power draw": "Công suất tiêu thụ",
  "{w} W limit": "giới hạn {w} W",
  Power: "Công suất",
  "{a} W of {b} W": "{a} W / {b} W",
  "Graphics clock": "Xung đồ họa",
  "Memory clock": "Xung bộ nhớ",
  Fan: "Quạt",
  "Video encode": "Mã hóa video",
  "Video decode": "Giải mã video",
  "Driver version": "Phiên bản driver",
  "Active time": "Thời gian bận",
  "Average response time": "Thời gian phản hồi trung bình",
  Capacity: "Dung lượng",
  "Transfer rate, read above and write below": "Tốc độ truyền, đọc ở trên và ghi ở dưới",
  "Read speed": "Tốc độ đọc",
  "Write speed": "Tốc độ ghi",
  "File system": "Hệ tệp",
  "Volume name": "Tên ổ",
  "Mount point": "Điểm gắn",
  Removable: "Tháo rời được",
  "Throughput, receive above and send below": "Thông lượng, nhận ở trên và gửi ở dưới",
  "Total received": "Tổng đã nhận",
  "Total sent": "Tổng đã gửi",
  "MAC address": "Địa chỉ MAC",
  "IPv4 address": "Địa chỉ IPv4",
  "IPv6 addresses": "Địa chỉ IPv6",
  "down {r}, up {t}": "xuống {r}, lên {t}",
  Name: "Tên",
  PID: "PID",
  CPU: "CPU",
  Parent: "Cha",
  User: "Người dùng",
  Virtual: "Bộ nhớ ảo",
  "Disk read": "Đọc đĩa",
  "Disk write": "Ghi đĩa",
  "Run time": "Đã chạy",
  "Filter by name, PID, user or path": "Lọc theo tên, PID, người dùng hoặc đường dẫn",
  "Filter processes": "Lọc tiến trình",
  "Group by name": "Gộp theo tên",
  Columns: "Cột",
  Pause: "Tạm dừng",
  Resume: "Tiếp tục",
  "Sort now": "Sắp xếp lại",
  groups: "nhóm",
  "Pause updates. Values stay as they are.": "Tạm dừng cập nhật. Số liệu giữ nguyên.",
  "Rows keep their place between sorts. Press to sort by the current column.": "Các dòng giữ nguyên vị trí giữa các lần sắp xếp. Bấm để sắp xếp theo cột hiện tại.",
  "Process details": "Chi tiết tiến trình",
  Started: "Bắt đầu lúc",
  "Memory (working set)": "Bộ nhớ (working set)",
  exited: "đã thoát",
  Path: "Đường dẫn",
  "Command line": "Dòng lệnh",
  "not readable": "không đọc được",
  "Turn off grouping to see one process's details.": "Tắt gộp nhóm để xem chi tiết từng tiến trình.",
  "Select a process to see its path, command line, parent and user.": "Chọn một tiến trình để xem đường dẫn, dòng lệnh, tiến trình cha và người dùng.",
  "Open dashboard": "Mở dashboard",
  "SysVane needs an update": "SysVane cần được cập nhật",
  "The window and the background program are different versions, so the window cannot talk to it safely. Update SysVane, then open it again.": "Cửa sổ và chương trình nền khác phiên bản nên không thể giao tiếp an toàn. Hãy cập nhật SysVane rồi mở lại.",
} satisfies Record<string, string>;

export type Msg = keyof typeof VI;

const KEY = "sysvane-lang";
const subs = new Set<() => void>();

const load = (): Lang => {
  const v = localStorage.getItem(KEY);
  return v === "en" || v === "vi" ? v : "auto";
};

let pref: Lang = load();

/** "auto" follows the Windows display language. */
export const resolved = (): "en" | "vi" => (pref === "auto" ? (navigator.language.toLowerCase().startsWith("vi") ? "vi" : "en") : pref);

export function t(en: Msg, vars?: Record<string, string | number>): string {
  const text = resolved() === "vi" ? VI[en] : en;
  return vars ? text.replace(/\{(\w+)\}/g, (_, k: string) => String(vars[k] ?? `{${k}}`)) : text;
}

const NEXT: Record<Lang, Lang> = { auto: "en", en: "vi", vi: "auto" };

export function useLang(): [Lang, () => void] {
  const lang = useSyncExternalStore(
    (f) => {
      subs.add(f);
      return () => subs.delete(f);
    },
    () => pref,
  );
  const cycle = () => {
    pref = NEXT[pref];
    localStorage.setItem(KEY, pref);
    subs.forEach((f) => f());
  };
  return [lang, cycle];
}
