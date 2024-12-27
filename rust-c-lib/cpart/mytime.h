typedef struct tm {
  int tm_sec;  /* 秒 */
  int tm_min;  /* 分钟 */
  int tm_hour;   /* 小时 */
  int tm_mday;   /* 日 */
  int tm_mon;  /* 月 */
  int tm_year;   /* 年 */
  int tm_wday;   /* 星期 */
  int tm_yday;   /* 一年中的第几天 */
  int tm_isdst;  /* 夏令时 */
} StructTM;
extern int mktime(StructTM*);
extern char* asctime(StructTM*);