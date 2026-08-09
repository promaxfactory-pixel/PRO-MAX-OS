import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import Card from "@/components/ui/Card";
import Button from "@/components/ui/Button";
import { invoke } from "@tauri-apps/api/core";
import { ArrowRight, Save } from "lucide-react";
import { useUIStore } from "@/stores/uiStore";
import type { Employee } from "@/types";

export default function EmployeeFormPage() {
  const { t } = useTranslation();
  const { id } = useParams();
  const navigate = useNavigate();
  const isEdit = Boolean(id);
  const addNotification = useUIStore((s) => s.addNotification);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [form, setForm] = useState({
    code: "", name: "", nationality: "", job: "", phone: "",
    id_number: "", date_of_birth: "", gender: "", marital_status: "", email: "",
    salary_milli: 0, basic_salary_milli: 0, housing_allowance_milli: 0,
    transport_allowance_milli: 0, food_allowance_milli: 0, other_allowances_milli: 0,
    allowances_milli: 0, overtime_rate_milli: 0,
    passport_no: "", passport_expiry: "", residence_expiry: "", visa_expiry: "",
    workpermit_expiry: "", insurance_expiry: "", insurance_policy_no: "",
    insurance_premium_milli: 0, ticket_allowance_milli: 0,
    contract_end: "", joining_date: "",
    bank_name: "", bank_account_no: "", sponsor_name: "", sponsor_id: "",
    notes: "",
  });

  useEffect(() => {
    if (isEdit) {
      setLoading(true);
      invoke<Employee>("get_employee", { id: Number(id) }).then((d) => {
        setForm({
          code: d.code || "", name: d.name || "", nationality: d.nationality || "",
          job: d.job || "", phone: d.phone || "",
          id_number: d.id_number || "", date_of_birth: d.date_of_birth || "",
          gender: d.gender || "", marital_status: d.marital_status || "", email: d.email || "",
          salary_milli: d.salary_milli || 0, basic_salary_milli: d.basic_salary_milli || 0,
          housing_allowance_milli: d.housing_allowance_milli || 0,
          transport_allowance_milli: d.transport_allowance_milli || 0,
          food_allowance_milli: d.food_allowance_milli || 0,
          other_allowances_milli: d.other_allowances_milli || 0,
          allowances_milli: d.allowances_milli || 0, overtime_rate_milli: d.overtime_rate_milli || 0,
          passport_no: d.passport_no || "", passport_expiry: d.passport_expiry || "",
          residence_expiry: d.residence_expiry || "", visa_expiry: d.visa_expiry || "",
          workpermit_expiry: d.workpermit_expiry || "", insurance_expiry: d.insurance_expiry || "",
          insurance_policy_no: d.insurance_policy_no || "",
          insurance_premium_milli: d.insurance_premium_milli || 0,
          ticket_allowance_milli: d.ticket_allowance_milli || 0,
          contract_end: d.contract_end || "", joining_date: d.joining_date || "",
          bank_name: d.bank_name || "", bank_account_no: d.bank_account_no || "",
          sponsor_name: d.sponsor_name || "", sponsor_id: d.sponsor_id || "",
          notes: d.notes || "",
        });
      }).catch((e: unknown) => addNotification({ title: t('common.error'), message: String(e), type: 'error' })).finally(() => setLoading(false));
    }
  }, [id, isEdit, t]);

  const set = (key: string, val: string | number | boolean) => setForm((f) => ({ ...f, [key]: val }));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!form.name.trim()) return addNotification({ id: crypto.randomUUID(), type: "warning", title: t("common.warning"), message: t("employeeForm.nameRequired") });
    setSaving(true);
    try {
      if (isEdit) {
        await invoke("update_employee", { id: Number(id), input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("employeeForm.saveSuccess") });
        navigate(`/hr/employees/${id}`);
      } else {
        const newId = await invoke("create_employee", { input: form });
        addNotification({ id: crypto.randomUUID(), type: "success", title: t("common.success"), message: t("employeeForm.saveSuccess") });
        navigate(`/hr/employees/${newId}`);
      }
    } catch (err: unknown) { addNotification({ id: crypto.randomUUID(), type: "error", title: t("common.error"), message: t("employeeForm.saveError") }); }
    finally { setSaving(false); }
  };

  if (loading) return <div className="flex items-center justify-center h-64"><div className="w-12 h-12 border-2 border-brand-800 border-t-gold-400 rounded-full animate-spin" /></div>;

  return (
    <div className="space-y-6">
      <div className="page-header">
        <div className="flex items-center gap-3">
          <button onClick={() => navigate('/hr/employees')} className="btn-ghost p-2"><ArrowRight className="w-5 h-5" /></button>
          <div>
            <h1 className="page-title">{isEdit ? t("employeeForm.editTitle") : t("employeeForm.newTitle")}</h1>
            <p className="page-subtitle">{isEdit ? t("employeeForm.editSubtitle", { name: form.name }) : t("employeeForm.newTitle")}</p>
          </div>
        </div>
      </div>

      <form onSubmit={handleSubmit}>
        <Card>
          <h3 className="section-title mb-4">{t("employeeForm.personalData")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("employeeForm.nameLabel")}</label>
              <input className="input-field" value={form.name} onChange={(e) => set("name", e.target.value)} required aria-label={t("employeeForm.nameAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.employeeCode")}</label>
              <input className="input-field" value={form.code} onChange={(e) => set("code", e.target.value)} aria-label={t("employeeForm.employeeCode")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.idNumber")}</label>
              <input className="input-field" value={form.id_number} onChange={(e) => set("id_number", e.target.value)} aria-label={t("employeeForm.idNumber")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.nationality")}</label>
              <input className="input-field" value={form.nationality} onChange={(e) => set("nationality", e.target.value)} aria-label={t("hr.nationality")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.job")}</label>
              <input className="input-field" value={form.job} onChange={(e) => set("job", e.target.value)} aria-label={t("hr.job")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.gender")}</label>
              <select className="input-field" value={form.gender} onChange={(e) => set("gender", e.target.value)} aria-label={t("employeeForm.gender")}>
                <option value="">{t("employeeForm.selectPlaceholder")}</option>
                <option value="ذكر">{t("employeeForm.male")}</option>
                <option value="أنثى">{t("employeeForm.female")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.maritalStatus")}</label>
              <select className="input-field" value={form.marital_status} onChange={(e) => set("marital_status", e.target.value)} aria-label={t("employeeForm.maritalStatus")}>
                <option value="">{t("employeeForm.selectPlaceholder")}</option>
                <option value="أعزب">{t("employeeForm.single")}</option>
                <option value="متزوج">{t("employeeForm.married")}</option>
                <option value="متزوجة">{t("employeeForm.marriedFemale")}</option>
              </select>
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.dateOfBirth")}</label>
              <input className="input-field" type="date" value={form.date_of_birth} onChange={(e) => set("date_of_birth", e.target.value)} aria-label={t("employeeForm.dateOfBirth")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.phone")}</label>
              <input className="input-field" value={form.phone} onChange={(e) => set("phone", e.target.value)} aria-label={t("hr.phone")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.email")}</label>
              <input className="input-field" type="email" value={form.email} onChange={(e) => set("email", e.target.value)} aria-label={t("employeeForm.email")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.joiningDate")}</label>
              <input className="input-field" type="date" value={form.joining_date} onChange={(e) => set("joining_date", e.target.value)} aria-label={t("employeeForm.joiningDate")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("employeeForm.salaryBenefits")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("employeeForm.grossSalary")}</label>
              <input className="input-field" type="number" value={form.salary_milli} onChange={(e) => set("salary_milli", Number(e.target.value))} aria-label={t("employeeForm.grossSalaryAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.basicSalary")}</label>
              <input className="input-field" type="number" value={form.basic_salary_milli} onChange={(e) => set("basic_salary_milli", Number(e.target.value))} aria-label={t("employeeForm.basicSalaryAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.housingAllowance")}</label>
              <input className="input-field" type="number" value={form.housing_allowance_milli} onChange={(e) => set("housing_allowance_milli", Number(e.target.value))} aria-label={t("employeeForm.housingAllowanceAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.transportAllowance")}</label>
              <input className="input-field" type="number" value={form.transport_allowance_milli} onChange={(e) => set("transport_allowance_milli", Number(e.target.value))} aria-label={t("employeeForm.transportAllowanceAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.foodAllowance")}</label>
              <input className="input-field" type="number" value={form.food_allowance_milli} onChange={(e) => set("food_allowance_milli", Number(e.target.value))} aria-label={t("employeeForm.foodAllowanceAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.otherAllowances")}</label>
              <input className="input-field" type="number" value={form.other_allowances_milli} onChange={(e) => set("other_allowances_milli", Number(e.target.value))} aria-label={t("employeeForm.otherAllowancesAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.totalAllowances")}</label>
              <input className="input-field" type="number" value={form.allowances_milli} onChange={(e) => set("allowances_milli", Number(e.target.value))} aria-label={t("employeeForm.totalAllowancesAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.overtimeRate")}</label>
              <input className="input-field" type="number" value={form.overtime_rate_milli} onChange={(e) => set("overtime_rate_milli", Number(e.target.value))} aria-label={t("employeeForm.overtimeRateAria")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("employeeForm.documents")}</h3>
          <div className="grid grid-cols-3 gap-6">
            <div className="input-group">
              <label className="input-label">{t("hr.passport")}</label>
              <input className="input-field" value={form.passport_no} onChange={(e) => set("passport_no", e.target.value)} aria-label={t("hr.passport")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.passportExpiry")}</label>
              <input className="input-field" type="date" value={form.passport_expiry} onChange={(e) => set("passport_expiry", e.target.value)} aria-label={t("hr.passportExpiry")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.residenceExpiry")}</label>
              <input className="input-field" type="date" value={form.residence_expiry} onChange={(e) => set("residence_expiry", e.target.value)} aria-label={t("hr.residenceExpiry")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.visaExpiry")}</label>
              <input className="input-field" type="date" value={form.visa_expiry} onChange={(e) => set("visa_expiry", e.target.value)} aria-label={t("hr.visaExpiry")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.workPermitExpiry")}</label>
              <input className="input-field" type="date" value={form.workpermit_expiry} onChange={(e) => set("workpermit_expiry", e.target.value)} aria-label={t("employeeForm.workPermitExpiry")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.insuranceExpiry")}</label>
              <input className="input-field" type="date" value={form.insurance_expiry} onChange={(e) => set("insurance_expiry", e.target.value)} aria-label={t("employeeForm.insuranceExpiry")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.insurancePolicyNo")}</label>
              <input className="input-field" value={form.insurance_policy_no} onChange={(e) => set("insurance_policy_no", e.target.value)} aria-label={t("employeeForm.insurancePolicyNo")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.insurancePremium")}</label>
              <input className="input-field" type="number" value={form.insurance_premium_milli} onChange={(e) => set("insurance_premium_milli", Number(e.target.value))} aria-label={t("employeeForm.insurancePremiumAria")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("hr.contractEnd")}</label>
              <input className="input-field" type="date" value={form.contract_end} onChange={(e) => set("contract_end", e.target.value)} aria-label={t("hr.contractEnd")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.annualFlightAllowance")}</label>
              <input className="input-field" type="number" value={form.ticket_allowance_milli} onChange={(e) => set("ticket_allowance_milli", Number(e.target.value))} aria-label={t("employeeForm.annualFlightAllowanceAria")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("employeeForm.bankSponsor")}</h3>
          <div className="grid grid-cols-2 gap-6">
            <div className="input-group">
              <label className="input-label">{t("employeeForm.bankName")}</label>
              <input className="input-field" value={form.bank_name} onChange={(e) => set("bank_name", e.target.value)} aria-label={t("employeeForm.bankName")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.bankAccountNo")}</label>
              <input className="input-field" value={form.bank_account_no} onChange={(e) => set("bank_account_no", e.target.value)} aria-label={t("employeeForm.bankAccountNo")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.sponsorName")}</label>
              <input className="input-field" value={form.sponsor_name} onChange={(e) => set("sponsor_name", e.target.value)} aria-label={t("employeeForm.sponsorName")} />
            </div>
            <div className="input-group">
              <label className="input-label">{t("employeeForm.sponsorId")}</label>
              <input className="input-field" value={form.sponsor_id} onChange={(e) => set("sponsor_id", e.target.value)} aria-label={t("employeeForm.sponsorId")} />
            </div>
          </div>
        </Card>

        <Card className="mt-4">
          <h3 className="section-title mb-4">{t("common.notes")}</h3>
          <div className="input-group">
            <textarea className="input-field" rows={3} value={form.notes} onChange={(e) => set("notes", e.target.value)} aria-label={t("common.notes")} />
          </div>
        </Card>

        <div className="flex justify-start gap-3 mt-4">
          <Button type="submit" loading={saving} icon={<Save className="w-4 h-4" />}>{isEdit ? t("employeeForm.saveChanges") : t("employeeForm.addEmployee")}</Button>
          <Button variant="outline" type="button" onClick={() => navigate('/hr/employees')}>{t("common.cancel")}</Button>
        </div>
      </form>
    </div>
  );
}
