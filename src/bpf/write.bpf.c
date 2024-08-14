#include "vmlinux.h"

#include <bpf/bpf_helpers.h>
#include <errno.h>

extern struct hid_bpf_ctx *hid_bpf_allocate_context(unsigned int hid_id) __ksym;
extern void hid_bpf_release_context(struct hid_bpf_ctx *ctx) __ksym;
extern int hid_bpf_hw_request(struct hid_bpf_ctx *ctx, const u8 *data,
                              size_t len, enum hid_report_type type,
                              enum hid_class_request reqtype) __ksym;

enum {
  MAX_BLOCK_SIZE = 256,
};

struct Block {
  u8 buf[MAX_BLOCK_SIZE];
};

struct {
  __uint(type, BPF_MAP_TYPE_ARRAY);
  __type(key, u32);
  __type(value, struct Block);
  __uint(max_entries, 1);
} array SEC(".maps");

struct Hdr {
  unsigned int hid_id;
  unsigned int data_size;
};

SEC("syscall") int write(struct Hdr *hdr) {
  int ret = 0;
  struct hid_bpf_ctx *hid_ctx = NULL;
  u32 key = 0;

  const struct Block *block = bpf_map_lookup_elem(&array, &key);
  if (!block) {
    ret = -EINVAL;
    goto exit;
  }

  hid_ctx = hid_bpf_allocate_context(hdr->hid_id);
  if (!hid_ctx) {
    ret = -EPERM;
    goto exit;
  }

  if (hdr->data_size >= MAX_BLOCK_SIZE) {
    ret = -EINVAL;
    goto exit;
  }

  ret = hid_bpf_hw_request(hid_ctx, block->buf, hdr->data_size,
                           HID_FEATURE_REPORT, HID_REQ_SET_REPORT);

exit:
  if (hid_ctx)
    hid_bpf_release_context(hid_ctx);

  return ret;
}

char _license[] SEC("license") = "GPL";
