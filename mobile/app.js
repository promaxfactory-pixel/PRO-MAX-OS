(function () {
  'use strict';

  const app = document.getElementById('app');
  let state = {
    token: localStorage.getItem('pmx_token') || null,
    user: JSON.parse(localStorage.getItem('pmx_user') || 'null'),
    view: 'home',
    unread: 0
  };

  /* ─── Utilities ─── */
  function $(sel, root) { return (root || document).querySelector(sel); }
  function el(tag, attrs, children) {
    const node = document.createElement(tag);
    if (attrs) for (const [k, v] of Object.entries(attrs)) {
      if (k === 'class') node.className = v;
      else if (k === 'text') node.textContent = v;
      else if (k === 'html') node.innerHTML = v;
      else if (k.startsWith('on') && typeof v === 'function') node.addEventListener(k.slice(2), v);
      else node.setAttribute(k, v);
    }
    (children || []).forEach((c) => { if (c) node.appendChild(typeof c === 'string' ? document.createTextNode(c) : c); });
    return node;
  }
  function esc(s) {
    return String(s == null ? '' : s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
  }
  function num(x) {
    const n = Number(x || 0);
    return n.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }
  function money(x) { return num(x) + ' ر.ع'; }
  function fmtDate(s) { return s ? String(s).replace('T', ' ').slice(0, 16) : '—'; }
  function pill(status) {
    const map = {
      paid: ['ok', 'مدفوعة'], unpaid: ['warn', 'غير مدفوعة'], partial: ['warn', 'مسددة جزئياً'],
      void: ['bad', 'ملغاة'], cancelled: ['bad', 'ملغاة'], draft: ['neutral', 'مسودة'],
      pending: ['warn', 'قيد الانتظار'], approved: ['ok', 'معتمدة'], rejected: ['bad', 'مرفوضة'],
      high: ['bad', 'عالية'], normal: ['neutral', 'عادية'], urgent: ['bad', 'عاجلة'],
      active: ['ok', 'نشط'], warning: ['warn', 'تنبيه']
    };
    const p = map[status] || ['neutral', status || ''];
    return '<span class="pill ' + p[0] + '">' + esc(p[1]) + '</span>';
  }
  function toast(msg, isError) {
    let t = document.getElementById('toast');
    t.textContent = msg;
    t.className = 'toast show' + (isError ? ' error' : '');
    clearTimeout(t._t);
    t._t = setTimeout(() => { t.className = 'toast'; }, 2600);
  }
  function storeAuth(token, user) {
    state.token = token; state.user = user;
    localStorage.setItem('pmx_token', token);
    localStorage.setItem('pmx_user', JSON.stringify(user));
  }
  function logout() {
    if (state.token) {
      api('/api/auth/logout', { method: 'POST', silent: true }).catch(function () {});
    }
    state.token = null; state.user = null;
    localStorage.removeItem('pmx_token');
    localStorage.removeItem('pmx_user');
    render();
  }

  /* ─── API client ─── */
  async function api(path, opts) {
    opts = opts || {};
    const headers = { 'Content-Type': 'application/json' };
    if (state.token) headers['Authorization'] = 'Bearer ' + state.token;
    let res;
    try {
      res = await fetch(path, { method: opts.method || 'GET', headers, body: opts.body ? JSON.stringify(opts.body) : undefined });
    } catch (e) {
      throw { _net: true, message: 'تعذر الاتصال بالخادم' };
    }
    let data = null;
    try { data = await res.json(); } catch (e) { /* empty body */ }
    if (res.status === 401 && state.token && !opts.silent) {
      state.token = null; state.user = null;
      localStorage.removeItem('pmx_token');
      localStorage.removeItem('pmx_user');
      render();
      throw { message: 'انتهت الجلسة، سجّل الدخول مجدداً' };
    }
    if (!res.ok) throw { message: (data && data.error) || 'خطأ غير متوقع (' + res.status + ')' };
    return data;
  }

  /* ─── App shell ─── */
  const TABS = [
    { id: 'home', label: 'الرئيسية', ico: '🏠' },
    { id: 'sales', label: 'العمليات', ico: '🧾' },
    { id: 'approvals', label: 'الموافقات', ico: '✅' },
    { id: 'alerts', label: 'التنبيهات', ico: '🔔' },
    { id: 'more', label: 'المزيد', ico: '☰' }
  ];

  function shellHeader(title, sub) {
    return el('div', { class: 'shell-header' }, [
      el('div', {}, [
        el('div', { class: 'title', text: title }),
        el('div', { class: 'sub', text: sub || 'مدير الجوال • PRO MAX OS' })
      ]),
      el('div', { class: 'header-actions' }, [
        el('button', { text: 'تحديث', onclick: render }),
        el('button', { text: 'خروج', onclick: logout })
      ])
    ]);
  }
  function shellNav(activeId) {
    return el('nav', { class: 'shell-nav' }, TABS.map((t) =>
      el('button', { class: t.id === activeId ? 'active' : '', onclick: () => go(t.id) }, [
        el('span', { class: 'nav-ico', text: t.ico }),
        el('span', { text: t.label }),
        t.id === 'alerts' && state.unread > 0 ? el('span', { class: 'badge', text: String(state.unread) }) : null
      ])
    ));
  }
  function openView(title, sub, activeId) {
    const body = el('div', { class: 'shell-body' });
    const v = el('div', { class: 'shell' }, [
      shellHeader(title, sub),
      body,
      shellNav(activeId)
    ]);
    app.innerHTML = '';
    app.appendChild(v);
    return body;
  }
  function go(view) { state.view = view; render(); }

  function render() {
    if (!state.token) { renderLogin(); return; }
    const v = state.view;
    if (v === 'home') return renderHome();
    if (v === 'sales') return renderSales();
    if (v === 'approvals') return renderApprovals();
    if (v === 'alerts') return renderAlerts();
    if (v === 'more') return renderMore();
    renderHome();
  }

  /* ─── Login ─── */
  function renderLogin() {
    const wrap = el('div', { class: 'login-wrap' }, [
      el('div', { class: 'login-card' }, [
        el('div', { class: 'login-logo' }, [
          el('img', { src: '/icons/icon-192.png', alt: 'PRO MAX OS' }),
          el('h1', { text: 'PRO MAX OS' }),
          el('p', { text: 'نظام إدارة الشركات المتكامل — مدير الجوال' })
        ]),
        el('div', { class: 'field' }, [el('label', { text: 'اسم المستخدم' }), el('input', { id: 'u', type: 'text', autocapitalize: 'off', autocomplete: 'username' })]),
        el('div', { class: 'field' }, [el('label', { text: 'كلمة المرور' }), el('input', { id: 'p', type: 'password', autocomplete: 'current-password' })]),
        el('div', { class: 'login-error', id: 'lerr' }),
        el('button', { class: 'btn-primary', id: 'lbtn', text: 'دخول' })
      ])
    ]);
    app.innerHTML = '';
    app.appendChild(wrap);
    const u = $('#u'), p = $('#p'), btn = $('#lbtn');
    const doLogin = async () => {
      if (!u.value || !p.value) { $('#lerr').textContent = 'أدخل اسم المستخدم وكلمة المرور'; return; }
      btn.disabled = true; btn.textContent = 'جارٍ الدخول...';
      try {
        const data = await api('/api/auth/login', { method: 'POST', body: { username: u.value.trim(), password: p.value }, silent: true });
        storeAuth(data.token, { username: data.user, full_name: data.full_name || data.user, role: data.role });
        state.view = 'home';
        render();
      } catch (e) {
        $('#lerr').textContent = e._net ? 'تعذر الاتصال بالخادم' : (e.message || 'خطأ في الدخول');
        btn.disabled = false; btn.textContent = 'دخول';
      }
    };
    btn.addEventListener('click', doLogin);
    p.addEventListener('keydown', (e) => { if (e.key === 'Enter') doLogin(); });
    setTimeout(() => u.focus(), 100);
  }

  /* ─── Home ─── */
  async function renderHome() {
    const body = openView('PRO MAX OS', (state.user && state.user.full_name) || '', 'home');
    body.appendChild(loading());
    try {
      const [kpis, dash] = await Promise.all([api('/api/kpis'), api('/api/dashboard')]);
      state.unread = kpis.unread_notifications || 0;
      const kpiCards = el('div', { class: 'kpis' }, [
        kpi('مبيعات اليوم', money(kpis.today_sales_omr), 'اليوم', kpis.today_sales_omr > 0 ? 'gold' : ''),
        kpi('مبيعات الشهر', money(kpis.month_sales_omr), 'الحركة الكاملة', 'gold'),
        kpi('مشتريات الشهر', money(kpis.month_purchases_omr), '', ''),
        kpi('مصاريف الشهر', money(kpis.month_expenses_omr), '', ''),
        kpi('الذمم المدينة', money(kpis.receivables_omr), kpis.unpaid_invoices + ' فاتورة غير مدفوعة', kpis.receivables_omr > 0 ? 'warn' : ''),
        kpi('الذمم الدائنة', money(kpis.payables_omr), '', kpis.payables_omr > 0 ? 'warn' : ''),
        kpi('موافقات معلقة', String(kpis.pending_approvals), 'بقيمة ' + money(kpis.pending_approval_amount_omr), kpis.pending_approvals > 0 ? 'warn' : ''),
        kpi('تنبيهات غير مقروءة', String(kpis.unread_notifications), kpis.low_stock_items + ' مخزون منخفض', kpis.unread_notifications > 0 ? 'warn' : '')
      ]);
      const recentPanel = el('div', { class: 'panel' }, [
        el('div', { class: 'panel-head' }, [
          el('h3', { text: 'أحدث الفواتير' }),
          el('a', { text: 'الكل', href: '#', onclick: (e) => { e.preventDefault(); state.view = 'sales'; render(); } })
        ]),
        recentList(dash.recent_invoices || [])
      ]);
      const view = el('div', {}, [
        kpiCards,
        el('div', { class: 'section-title', text: 'نظرة عامة' }),
        recentPanel
      ]);
      body.innerHTML = '';
      body.appendChild(view);
    } catch (e) {
      body.innerHTML = '';
      body.appendChild(errorView(e.message, renderHome));
    }
  }
  function kpi(label, value, note, tone) {
    return el('div', { class: 'kpi ' + (tone || '') }, [
      el('div', { class: 'k-label', text: label }),
      el('div', { class: 'k-value', text: value }),
      note ? el('div', { class: 'k-note', text: note }) : null
    ]);
  }
  function recentList(items) {
    if (!items.length) return el('div', { class: 'empty', html: '<div class="big">📄</div>لا فواتير بعد' });
    const wrap = el('div', {});
    items.forEach((inv) => wrap.appendChild(
      el('div', { class: 'list-item', onclick: () => openInvoice(inv.id) }, [
        el('div', { class: 'li-main' }, [
          el('div', { class: 'li-title', text: inv.invoice_no || ('#' + inv.id) }),
          el('div', { class: 'li-sub', text: inv.customer_name + ' • ' + fmtDate(inv.date) })
        ]),
        el('div', { class: 'li-side' }, [
          el('div', { class: 'li-amount', text: money(inv.total_omr) }),
          el('div', { class: 'li-meta', html: pill(inv.status) })
        ])
      ])
    ));
    return wrap;
  }
  function loading() {
    const s = el('div', { class: 'spinner' });
    return s;
  }
  function errorView(msg, retry) {
    return el('div', { class: 'empty' }, [
      el('div', { class: 'big', text: '⚠️' }),
      el('div', { text: msg }),
      retry ? el('button', { text: 'إعادة المحاولة', onclick: retry, style: 'margin-top:12px' }) : null
    ]);
  }

  /* ─── Sales (invoices + purchases + expenses) ─── */
  async function renderSales() {
    const bodyEl = openView('العمليات', 'الفواتير • المشتريات • المصاريف', 'sales');
    const seg = el('div', { class: 'btn-row', style: 'margin-bottom:12px' }, [
      el('button', { class: 'btn-ghost', id: 'seg-inv', text: 'الفواتير' }),
      el('button', { class: 'btn-ghost', id: 'seg-pur', text: 'المشتريات' }),
      el('button', { class: 'btn-ghost', id: 'seg-exp', text: 'المصاريف' })
    ]);
    const listWrap = el('div', {});
    bodyEl.appendChild(seg);
    bodyEl.appendChild(listWrap);

    let mode = 'invoices';
    const active = (m) => { seg.querySelectorAll('button').forEach((b) => b.classList.remove('active')); seg.querySelector('#seg-' + (m === 'invoices' ? 'inv' : m === 'purchases' ? 'pur' : 'exp')).classList.add('active'); };
    $('#seg-inv').addEventListener('click', () => { mode = 'invoices'; active(mode); load(); });
    $('#seg-pur').addEventListener('click', () => { mode = 'purchases'; active(mode); load(); });
    $('#seg-exp').addEventListener('click', () => { mode = 'expenses'; active(mode); load(); });

    const load = async () => {
      listWrap.innerHTML = '';
      listWrap.appendChild(loading());
      try {
        if (mode === 'invoices') {
          const invs = await api('/api/invoices?limit=100');
          renderInvoiceList(listWrap, invs);
        } else if (mode === 'purchases') {
          const purs = await api('/api/purchases?limit=100');
          listWrap.appendChild(purchasesView(purs));
        } else {
          const exps = await api('/api/expenses?limit=100');
          listWrap.appendChild(el('div', { style: 'margin-bottom:10px' }, [
            el('button', { class: 'btn-approve', text: '+ إضافة مصروف', onclick: () => showNewExpense(load) })
          ]));
          listWrap.appendChild(expensesView(exps));
        }
      } catch (e) { listWrap.appendChild(errorView(e.message, load)); }
    };
    active('invoices');
    load();
  }
  function renderInvoiceList(wrap, invs) {
    if (!invs.length) { wrap.appendChild(el('div', { class: 'empty', html: '<div class="big">🧾</div>لا توجد فواتير' })); return; }
    const list = el('div', {});
    invs.forEach((inv) => list.appendChild(
      el('div', { class: 'list-item', onclick: () => openInvoice(inv.id) }, [
        el('div', { class: 'li-main' }, [
          el('div', { class: 'li-title', text: inv.invoice_no || ('#' + inv.id) }),
          el('div', { class: 'li-sub', text: inv.customer_name + ' • ' + fmtDate(inv.date) })
        ]),
        el('div', { class: 'li-side' }, [
          el('div', { class: 'li-amount', text: money(inv.total_omr) }),
          el('div', { class: 'li-meta', html: pill(inv.status) })
        ])
      ])
    ));
    wrap.appendChild(list);
  }
  function purchasesView(purs) {
    if (!purs.length) return el('div', { class: 'empty', html: '<div class="big">📦</div>لا توجد مشتريات' });
    const list = el('div', {});
    purs.forEach((p) => list.appendChild(
      el('div', { class: 'list-item' }, [
        el('div', { class: 'li-main' }, [
          el('div', { class: 'li-title', text: p.pur_no || ('#' + p.id) }),
          el('div', { class: 'li-sub', text: p.supplier_name + ' • ' + fmtDate(p.date) })
        ]),
        el('div', { class: 'li-side' }, [
          el('div', { class: 'li-amount', text: money(p.total_omr) }),
          el('div', { class: 'li-meta', html: pill(p.status) })
        ])
      ])
    ));
    return list;
  }
  function expensesView(exps) {
    if (!exps.length) return el('div', { class: 'empty', html: '<div class="big">💸</div>لا توجد مصاريف' });
    const list = el('div', {});
    exps.forEach((e) => list.appendChild(
      el('div', { class: 'list-item' }, [
        el('div', { class: 'li-main' }, [
          el('div', { class: 'li-title', text: e.category || 'مصروف' }),
          el('div', { class: 'li-sub', text: (e.exp_no || '') + ' • ' + fmtDate(e.date) })
        ]),
        el('div', { class: 'li-side' }, [
          el('div', { class: 'li-amount', text: money(e.amount_omr) }),
          el('div', { class: 'li-meta', html: pill(e.status) })
        ])
      ])
    ));
    return list;
  }

  function showNewExpense(onCreated) {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [
      el('h2', { text: 'إضافة مصروف' }),
      el('div', { class: 'field' }, [el('label', { text: 'المبلغ (ريال) *' }), el('input', { id: 'xamt', type: 'number', step: '0.001', min: '0', inputmode: 'decimal' })]),
      el('div', { class: 'field' }, [el('label', { text: 'التاريخ' }), el('input', { id: 'xdate', type: 'date' })]),
      el('div', { class: 'field' }, [el('label', { text: 'التصنيف' }), el('input', { id: 'xcat', type: 'text' })]),
      el('div', { class: 'field' }, [el('label', { text: 'طريقة الدفع' }), el('select', { id: 'xmethod' }, [
        el('option', { value: 'cash', text: 'نقداً' }),
        el('option', { value: 'bank', text: 'بنك' }),
        el('option', { value: 'card', text: 'بطاقة' }),
        el('option', { value: 'cheque', text: 'شيك' })
      ])]),
      el('div', { class: 'field' }, [el('label', { text: 'المورد/الجهة' }), el('input', { id: 'xven', type: 'text' })]),
      el('div', { class: 'field' }, [el('label', { text: 'المرجع' }), el('input', { id: 'xref', type: 'text' })]),
      el('div', { class: 'field' }, [el('label', { text: 'ملاحظات' }), el('input', { id: 'xnotes', type: 'text' })]),
      el('div', { class: 'btn-row' }, [
        el('button', { class: 'btn-approve', id: 'xsave', text: 'حفظ' }),
        el('button', { class: 'btn-reject', text: 'إلغاء', onclick: () => backdrop.remove() })
      ])
    ]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
    const dateInput = $('#xdate');
    dateInput.value = new Date().toISOString().slice(0, 10);
    $('#xsave').addEventListener('click', async () => {
      const amt = parseFloat($('#xamt').value);
      if (!isFinite(amt) || amt <= 0) { toast('أدخل مبلغاً صحيحاً', true); return; }
      const btn = $('#xsave');
      btn.disabled = true; btn.textContent = 'جارٍ الحفظ...';
      try {
        await api('/api/expenses', { method: 'POST', body: {
          amount_omr: amt,
          date: dateInput.value || null,
          category: $('#xcat').value.trim() || null,
          method: $('#xmethod').value,
          vendor: $('#xven').value.trim() || null,
          reference: $('#xref').value.trim() || null,
          notes: $('#xnotes').value.trim() || null
        } });
        backdrop.remove();
        toast('تمت إضافة المصروف');
        onCreated();
      } catch (e) {
        toast(e.message, true);
        btn.disabled = false; btn.textContent = 'حفظ';
      }
    });
  }

  /* ─── Invoice detail ─── */
  async function openInvoice(id) {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [el('div', { class: 'spinner' })]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
    try {
      const inv = await api('/api/invoices/' + id);
      sheet.innerHTML = '';
      sheet.appendChild(el('h2', { text: inv.invoice_no || ('فاتورة #' + inv.id) }));
      const rows = [
        ['الحالة', pill(inv.status)],
        ['العميل', esc(inv.customer_name || '—')],
        ['الرقم الضريبي', esc(inv.customer_vat || '—')],
        ['التاريخ', fmtDate(inv.date)],
        ['الإجمالي قبل الضريبة', money(inv.net_omr)],
        ['الضريبة', money(inv.vat_omr)],
        ['الإجمالي', money(inv.total_omr)],
        ['المدفوع', money(inv.paid_omr)]
      ];
      const dl = el('dl', { class: 'detail' });
      rows.forEach(([k, v]) => dl.appendChild(el('div', { class: 'detail-row', html: '<dt>' + esc(k) + '</dt><dd>' + v + '</dd>' })));
      sheet.appendChild(dl);
      if (inv.lines && inv.lines.length) {
        sheet.appendChild(el('div', { class: 'section-title', text: 'الأصناف' }));
        const lines = el('div', {});
        inv.lines.forEach((l) => lines.appendChild(
          el('div', { class: 'list-item' }, [
            el('div', { class: 'li-main' }, [
              el('div', { class: 'li-title', text: l.product || 'صنف' }),
              el('div', { class: 'li-sub', text: 'الكمية: ' + l.qty + ' × ' + money(l.unit_price_omr) })
            ]),
            el('div', { class: 'li-side' }, [el('div', { class: 'li-amount', text: money(l.total_omr) })])
          ])
        ));
        sheet.appendChild(lines);
      }
      if (inv.notes) sheet.appendChild(el('div', { class: 'section-title', text: 'ملاحظات: ' + esc(inv.notes) }));
      sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    } catch (e) {
      sheet.innerHTML = '';
      sheet.appendChild(el('h2', { text: 'خطأ' }));
      sheet.appendChild(el('div', { class: 'empty', text: e.message }));
      sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    }
  }

  /* ─── Approvals ─── */
  async function renderApprovals() {
    const body = openView('الموافقات', 'طلبات بانتظار القرار', 'approvals');
    body.appendChild(loading());
    try {
      const apps = await api('/api/approvals?status=pending&limit=100');
      body.innerHTML = '';
      if (!apps.length) { body.appendChild(el('div', { class: 'empty', html: '<div class="big">✅</div>لا توجد موافقات معلقة' })); return; }
      apps.forEach((a) => body.appendChild(approvalCard(a)));
    } catch (e) { body.innerHTML = ''; body.appendChild(errorView(e.message, renderApprovals)); }
  }
  function approvalCard(a) {
    const card = el('div', { class: 'panel' }, [
      el('div', { class: 'panel-head' }, [
        el('h3', { text: (a.entity_number || (a.request_type + ' #' + a.entity_id)) }),
        el('span', { html: pill(a.priority) })
      ]),
      el('div', { class: 'list-item' }, [
        el('div', { class: 'li-main' }, [
          el('div', { class: 'li-title', text: a.description || a.request_type }),
          el('div', { class: 'li-sub', text: 'طلب: ' + a.requested_by + ' • ' + fmtDate(a.requested_at) })
        ]),
        a.amount_omr != null ? el('div', { class: 'li-side' }, [el('div', { class: 'li-amount', text: money(a.amount_omr) })]) : null
      ]),
      el('div', { class: 'btn-row' }, [
        el('button', { class: 'btn-approve', text: 'اعتماد', onclick: () => decide(a, 'approve') }),
        el('button', { class: 'btn-reject', text: 'رفض', onclick: () => decide(a, 'reject') })
      ])
    ]);
    return card;
  }
  function decide(a, decision) {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [
      el('h2', { text: decision === 'approve' ? 'اعتماد الطلب' : 'رفض الطلب' }),
      el('div', { class: 'field' }, [el('label', { text: 'ملاحظة (اختياري)' }), el('textarea', { id: 'dreason', rows: 3, placeholder: 'سبب القرار...' })]),
      el('div', { class: 'btn-row' }, [
        el('button', { class: 'btn-ghost', text: 'إلغاء', onclick: () => backdrop.remove() }),
        el('button', { class: decision === 'approve' ? 'btn-approve' : 'btn-reject', text: 'تأكيد', onclick: async () => {
          try {
            await api('/api/approvals/' + a.id + '/decide', { method: 'POST', body: { decision: decision, reason: $('#dreason').value || null } });
            toast(decision === 'approve' ? 'تم الاعتماد' : 'تم الرفض');
            backdrop.remove();
            renderApprovals();
          } catch (e) { toast(e.message, true); }
        } })
      ])
    ]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
  }

  /* ─── Alerts & notifications ─── */
  async function renderAlerts() {
    const body = openView('التنبيهات', 'التنبيهات والإشعارات', 'alerts');
    body.appendChild(loading());
    try {
      const [alerts, notifs] = await Promise.all([api('/api/alerts'), api('/api/notifications?limit=50')]);
      state.unread = notifs.filter((n) => n.read_status === 'unread').length;

      if (alerts.low_stock && alerts.low_stock.length) {
        body.appendChild(el('div', { class: 'section-title', text: 'مخزون منخفض (' + alerts.low_stock.length + ')' }));
        alerts.low_stock.forEach((s) => body.appendChild(
          el('div', { class: 'list-item' }, [
            el('div', { class: 'li-main' }, [
              el('div', { class: 'li-title', text: s.name || ('صنف #' + s.item_id) }),
              el('div', { class: 'li-sub', text: 'الكمية: ' + num(s.quantity) + ' / حد الطلب: ' + num(s.reorder_level) })
            ]),
            el('div', { class: 'li-side', html: pill('warning') })
          ])
        ));
      }
      if (alerts.overdue_invoices && alerts.overdue_invoices.length) {
        body.appendChild(el('div', { class: 'section-title', text: 'فواتير متأخرة (' + alerts.overdue_invoices.length + ')' }));
        alerts.overdue_invoices.forEach((o) => body.appendChild(
          el('div', { class: 'list-item' }, [
            el('div', { class: 'li-main' }, [
              el('div', { class: 'li-title', text: o.invoice_no || ('#' + o.id) }),
              el('div', { class: 'li-sub', text: o.customer + ' • ' + fmtDate(o.date) })
            ]),
            el('div', { class: 'li-side' }, [el('div', { class: 'li-amount', text: money(o.due_omr) })])
          ])
        ));
      }
      if (alerts.expiring_renewals && alerts.expiring_renewals.length) {
        body.appendChild(el('div', { class: 'section-title', text: 'تجديدات قريبة (' + alerts.expiring_renewals.length + ')' }));
        alerts.expiring_renewals.forEach((r) => body.appendChild(
          el('div', { class: 'list-item' }, [
            el('div', { class: 'li-main' }, [
              el('div', { class: 'li-title', text: r.name || ('تجديد #' + r.id) }),
              el('div', { class: 'li-sub', text: 'تنتهي: ' + fmtDate(r.expiry_date) })
            ]),
            el('div', { class: 'li-side', html: pill('warning') })
          ])
        ));
      }

      body.appendChild(el('div', { class: 'section-title', text: 'الإشعارات' }));
      if (!notifs.length) { body.appendChild(el('div', { class: 'empty', html: '<div class="big">🔕</div>لا توجد إشعارات' })); return; }
      notifs.forEach((n) => body.appendChild(
        el('div', { class: 'list-item', onclick: () => markRead(n) }, [
          el('div', { class: 'li-main' }, [
            el('div', { class: 'li-title', text: n.title, style: n.read_status === 'unread' ? 'font-weight:800' : '' }),
            el('div', { class: 'li-sub', text: n.message + ' • ' + fmtDate(n.created_at) })
          ]),
          el('div', { class: 'li-side', html: pill(n.severity) })
        ])
      ));
    } catch (e) { body.innerHTML = ''; body.appendChild(errorView(e.message, renderAlerts)); }
  }
  async function markRead(n) {
    if (n.read_status === 'read') return;
    try { await api('/api/notifications/' + n.id + '/read', { method: 'POST' }); toast('تمت القراءة'); renderAlerts(); } catch (e) { toast(e.message, true); }
  }

  /* ─── More ─── */
  async function renderMore() {
    const body = openView('المزيد', (state.user && state.user.full_name) || '', 'more');
    body.appendChild(el('div', {}, [
      el('div', { class: 'menu-grid' }, [
        menuCard('📦', 'المنتجات', 'قائمة الأصناف والمخزون', () => showList('products')),
        menuCard('👥', 'العملاء', 'أرصدة العملاء', () => showList('customers')),
        menuCard('🧾', 'الفواتير', 'كل الفواتير', () => { state.view = 'sales'; render(); }),
        menuCard('📋', 'سجل النشاط', 'آخر الحركات', () => showActivity()),
        menuCard('🏢', 'بيانات الشركة', 'معلومات المنشأة', () => showCompany()),
        menuCard('🔑', 'تغيير كلمة المرور', 'تحديث كلمة المرور', () => showChangePassword())
      ]),
      el('div', { class: 'section-title', text: 'حساب' }),
      el('div', { class: 'panel' }, [
        el('div', { class: 'list-item', onclick: logout }, [
          el('div', { class: 'li-main' }, [el('div', { class: 'li-title', text: 'تسجيل الخروج' })]),
          el('div', { class: 'li-side', text: '↩' })
        ])
      ])
    ]));
  }
  function menuCard(ico, title, sub, onclick) {
    return el('button', { class: 'menu-card', onclick }, [
      el('div', { class: 'mi', text: ico }),
      el('div', { class: 'mt', text: title }),
      el('div', { class: 'ms', text: sub })
    ]);
  }

  async function showList(kind) {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [el('div', { class: 'spinner' })]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
    sheet.innerHTML = '';
    sheet.appendChild(el('h2', { text: kind === 'products' ? 'المنتجات' : 'العملاء' }));
    sheet.appendChild(el('div', { class: 'field search' }, [el('input', { id: 'sbox', type: 'search', placeholder: 'بحث...' })]));
    const listWrap = el('div', {});
    sheet.appendChild(listWrap);
    sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    const load = async (q) => {
      listWrap.innerHTML = '';
      listWrap.appendChild(loading());
      try {
        const items = kind === 'products' ? await api('/api/products') : await api('/api/customers');
        const filtered = q ? items.filter((i) => JSON.stringify(i).toLowerCase().includes(q.toLowerCase())) : items;
        if (!filtered.length) { listWrap.appendChild(el('div', { class: 'empty', text: 'لا نتائج' })); return; }
        const list = el('div', {});
        filtered.forEach((i) => list.appendChild(
          el('div', { class: 'list-item' }, [
            el('div', { class: 'li-main' }, [
              el('div', { class: 'li-title', text: i.name_ar || i.name_en || i.name || ('#' + i.id) }),
              el('div', { class: 'li-sub', text: kind === 'products'
                ? 'السعر: ' + money(i.price_omr) + ' • المخزون: ' + num(i.stock)
                : (i.phone || '') + ' • حد ائتماني: ' + money(i.credit_limit_omr) })
            ]),
            el('div', { class: 'li-side' }, kind === 'customers' ? [el('div', { class: 'li-amount', text: money(i.balance_omr) })] : [el('div', { class: 'li-meta', text: num(i.stock) + ' قطعة' })])
          ])
        ));
        listWrap.appendChild(list);
      } catch (e) { listWrap.appendChild(errorView(e.message, () => load(q))); }
    };
    $('#sbox').addEventListener('input', (e) => load(e.target.value));
    load('');
  }

  async function showActivity() {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [el('div', { class: 'spinner' })]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
    try {
      const acts = await api('/api/activity?limit=50');
      sheet.innerHTML = '';
      sheet.appendChild(el('h2', { text: 'سجل النشاط' }));
      const list = el('div', {});
      (acts || []).forEach((a) => list.appendChild(
        el('div', { class: 'list-item' }, [
          el('div', { class: 'li-main' }, [
            el('div', { class: 'li-title', text: (a.user || '') + ' • ' + (a.action || '') }),
            el('div', { class: 'li-sub', text: (a.entity || '') + (a.reason ? ' — ' + a.reason : '') })
          ]),
          el('div', { class: 'li-side' }, [el('div', { class: 'li-meta', text: fmtDate(a.ts) })])
        ])
      ));
      sheet.appendChild(list);
      sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    } catch (e) {
      sheet.innerHTML = '';
      sheet.appendChild(el('h2', { text: 'خطأ' }));
      sheet.appendChild(el('div', { class: 'empty', text: e.message }));
      sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    }
  }

  async function showCompany() {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [el('div', { class: 'spinner' })]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
    try {
      const c = await api('/api/company');
      sheet.innerHTML = '';
      sheet.appendChild(el('h2', { text: 'بيانات الشركة' }));
      const dl = el('dl', {});
      [['الاسم', c.name], ['المصنع', c.factory_name], ['ضريبة القيمة المضافة', (c.default_vat_pct != null ? c.default_vat_pct : 0) + '%'], ['الهاتف', c.phone], ['البريد', c.email], ['العنوان', c.address], ['الرقم الضريبي', c.vat_number]].forEach(([k, v]) =>
        dl.appendChild(el('div', { class: 'detail-row', html: '<dt>' + esc(k) + '</dt><dd>' + esc(v || '—') + '</dd>' }))
      );
      sheet.appendChild(dl);
      sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    } catch (e) {
      sheet.innerHTML = '';
      sheet.appendChild(el('h2', { text: 'خطأ' }));
      sheet.appendChild(el('div', { class: 'empty', text: e.message }));
      sheet.appendChild(el('button', { class: 'close', text: 'إغلاق', onclick: () => backdrop.remove() }));
    }
  }

  function showChangePassword() {
    const backdrop = el('div', { class: 'modal-backdrop' });
    const sheet = el('div', { class: 'modal-sheet' }, [
      el('h2', { text: 'تغيير كلمة المرور' }),
      el('div', { class: 'field' }, [el('label', { text: 'كلمة المرور الحالية' }), el('input', { id: 'cp1', type: 'password' })]),
      el('div', { class: 'field' }, [el('label', { text: 'كلمة المرور الجديدة' }), el('input', { id: 'cp2', type: 'password' })]),
      el('div', { class: 'field' }, [el('label', { text: 'تأكيد كلمة المرور الجديدة' }), el('input', { id: 'cp3', type: 'password' })]),
      el('div', { class: 'btn-row' }, [
        el('button', { class: 'btn-ghost', text: 'إلغاء', onclick: () => backdrop.remove() }),
        el('button', { class: 'btn-primary', text: 'حفظ', onclick: async () => {
          const oldP = $('#cp1').value, newP = $('#cp2').value, conf = $('#cp3').value;
          if (!oldP || !newP) { toast('أدخل كلمة المرور', true); return; }
          if (newP.length < 6) { toast('كلمة المرور الجديدة قصيرة جداً', true); return; }
          if (newP !== conf) { toast('تأكيد كلمة المرور غير متطابق', true); return; }
          try {
            await api('/api/auth/change-password', { method: 'POST', body: { current_password: oldP, new_password: newP } });
            toast('تم تغيير كلمة المرور');
            backdrop.remove();
            logout();
          } catch (e) { toast(e.message, true); }
        } })
      ])
    ]);
    backdrop.appendChild(sheet);
    document.body.appendChild(backdrop);
    backdrop.addEventListener('click', (e) => { if (e.target === backdrop) backdrop.remove(); });
  }

  render();
})();
