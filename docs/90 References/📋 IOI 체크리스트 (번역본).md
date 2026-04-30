출처  : https://ioi.github.io/checklist/

---
이 문서는 계속 업데이트 중인 작업 문서이며, 
IOI(국제정보올림피아드)를 개최할 때 고려해야 할 기술적인 사항들을 정리한 것이다.

---

# **🌐 대회 네트워크 인프라**

IOI 네트워크에는 다음과 같은 필수 및 권장 사항이 있다:

### **✅ 필수**

- 참가자 PC와 외부 인터넷 간 통신 차단
- 충분한 네트워크 속도:
    - 참가자 PC: 1Gbps
    - 채점 워커: 1Gbps
    - 서버: 가능하면 10Gbps
    - 스위치 간 백본: 가능하면 10Gbps

### **💡 권장**

- 멀티캐스트 지원 (이미징용)
- 직관적인 IP 주소 체계
- 로컬 DNS 서버 (hosts 파일 동기화는 번거로움)

---

## **🔒 참가자 간 통신 차단 방법**

- 각 PC를 개별 VLAN에 배치
- VLAN 간 라우팅은:
    - L3 스위치 (추천)
    - 또는 서버가 모든 VLAN에 직접 연결 (802.1q)

👉 L3 스위치가 좋은 이유:

- 멀티캐스트 지원 가능

---

### **🔸 Private VLAN (Cisco)**

- 동일 네트워크에서도 서로 통신 불가
- 지정된 서버만 접근 가능
- 멀티캐스트도 지원

---

## **🧠 IP 주소 설계**

- DHCP 커스터마이징 필요
- 방법:
    - 스위치 포트 기반
    - VLAN 기반

⚠️ 주의:

- DHCP IP 고정 시, lease 만료 전까지 재사용 불가
- 장비 이동 시 lease flush 필요

---

## **🔥 보안 및 안정성**

- 외부 서비스 (웹사이트, 랭킹)는 내부 네트워크와 분리
- DDoS 공격이 대회에 영향 주면 안 됨
- 스위치 간 연결은 반드시 이중화

---

# **💻 데스크탑 환경 설정**

## **🧾 하드웨어**

- RAM 최소 16GB
- 동일 기종 권장
- 키보드: ISO 레이아웃
- 마우스: 좌우 대칭형

---

## **⚙️ BIOS 설정**

- 부팅 순서: 네트워크 → 로컬 디스크
- 외부 부팅 차단
- BIOS 비밀번호 설정
- WiFi / Bluetooth 등 비활성화

⚠️ 일부 HP BIOS는 실패 시 메뉴로 진입 → 비활성화 필요

---

## **🧪 사전 테스트**

- memtest86
- badblocks
- smartctl

👉 문제 있는 장비는 사용 금지

---

# **🧑‍💻 소프트웨어 설정**

## **🔒 보안 설정**

- 사용자 그룹 제거
- swap 비활성화
- USB 차단:
    - `/media` 권한 제한 또는
    - usb_storage 모듈 블랙리스트
- NetworkManager 제거 → ifupdown 사용
- suid 프로그램 최소화

---

## **🧰 기타 설정**

- pcspkr 비활성화 (삐 소리 제거)
- remote syslog 설정
- ASLR 비활성화 (`randomize_va_space = 0`)
- stack size 조정
- TMPDIR → `$HOME/tmp/`

---

## **📦 배포 방식**

- Debian 패키지 활용
- 로컬 apt repository 구성
- Git / Ansible 사용 가능

---

# **⚙️ 실행 시간 안정성 (Determinism)**

- 단일 CPU에서 실행
- CPU governor → performance
- 터보부스트 비활성화
- hugepage 비활성화
- hyperthreading 비활성화 고려
- P-core만 사용 (Intel hybrid CPU)

---

## **☕ Java 실행 옵션**

```
-Xbatch -XX:+UseSerialGC -XX:-TieredCompilation -XX:CICompilerCount=1
```

---

# **🌍 웹 서버**

- nginx 로드밸런서 사용
- `ip_hash` 대신:

```
hash $remote_addr consistent;
```

- 캐시 방지 설정:

```
Cache-Control: no-store, no-cache
```

- DNS TTL 짧게 설정

---

# **💾 대량 이미지 배포**

- udpcast / CloneZilla 사용

### **성능 개선 방법**

- LZ4 / Snappy 압축
- 디스크 파티션 분리:
    - SYSTEM (읽기 전용)
    - USER

### **⚠️ 경험 사례**

- 전체 동시 이미지 → 느림
- 1/4씩 나누면 빠름 (15분 이내 완료)

---

# **🧑‍🔧 워커 수**

공식:

```
학생 수 / 20 × 3
```

예:

- 300명 → 최소 45 워커

---

# **🗄️ 데이터베이스**

- PostgreSQL replication (hot standby)
- pg_dump 시 주의:
    - slave에서 실패 가능 → replication 중지 후 덤프

---

# **💽 백업**

- 10분마다 rsync
- 옵션:
    - `--link-dest`
    - `--max-size`
    - `.cache` 제외

---

# **🧪 사전 테스트 항목**

- 전체 대회 시뮬레이션
- 무한루프 테스트
- 테스트케이스 변경 후 재채점

---

# **🧑‍💻 장애 대응**

- PC 고장 → 교체 절차
- 워커 제거 테스트
- 정전 대비

---

# **📣 대회 운영**

- 공지사항 웹으로만 전달
- 음식: 소리 없는 간식
- 제출 마감 직전 채점 지연 안내

---

# **⚖️ 이의제기 (Appeals)**

- 로컬 제출 수집
- 재채점
- ProxyService 재시작
- DB 백업
- 테스트 데이터 공개

---

# **🧾 조직 운영**

- 문제 비밀 유지
- 이벤트 로그 기록
- 규정 숙지

---

# **🌐 번역 네트워크**

- DHCP 최소 1000~4000 IP
- WiFi 2.4GHz 별도 구성
- NAT 지양

---

# **🖨️ 번역 출력**

- 최소 3대 프린터 (40ppm 이상)
- 6000장 이상 출력 가능

---

# **🧠 기타**

- 국기 비율 주의
- Java 메모리 충분히 확보 (2GB 권장)

---

# **☁️ 온라인 IOI (2020~2021)**

- AWS bare metal → 더 안정적
- Global Accelerator → 지연 감소
- VPN: Tinc 사용
- Ansible 활용

---
