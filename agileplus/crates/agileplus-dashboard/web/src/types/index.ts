/**
 * Component Type Definitions
 * Shared types for foundation, layout, and page components
 */

// ============================================================================
// Foundation Component Types
// ============================================================================

export interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost' | 'destructive';
  size?: 'sm' | 'md' | 'lg';
  disabled?: boolean;
  onClick?: (e: React.MouseEvent<HTMLButtonElement>) => void;
  className?: string;
  children: React.ReactNode;
  type?: 'button' | 'submit' | 'reset';
  ariaLabel?: string;
}

export interface InputProps {
  type?: 'text' | 'email' | 'password' | 'number' | 'date' | 'time';
  placeholder?: string;
  value?: string;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  disabled?: boolean;
  error?: string;
  label?: string;
  required?: boolean;
  className?: string;
  ariaLabel?: string;
  ariaDescribedBy?: string;
}

export interface SelectOption {
  value: string | number;
  label: string;
  disabled?: boolean;
}

export interface SelectProps {
  options: SelectOption[];
  value?: string | number;
  onChange?: (value: string | number) => void;
  placeholder?: string;
  label?: string;
  disabled?: boolean;
  error?: string;
  className?: string;
  ariaLabel?: string;
}

export interface CheckboxProps {
  checked?: boolean;
  onChange?: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
  required?: boolean;
  className?: string;
  ariaLabel?: string;
}

export interface RadioProps {
  value: string;
  checked?: boolean;
  onChange?: (value: string) => void;
  label?: string;
  disabled?: boolean;
  className?: string;
  ariaLabel?: string;
}

export interface ToggleProps {
  checked?: boolean;
  onChange?: (checked: boolean) => void;
  label?: string;
  icon?: React.ReactNode;
  disabled?: boolean;
  className?: string;
  ariaLabel?: string;
  ariaPressed?: boolean;
}

// ============================================================================
// Layout Component Types
// ============================================================================

export interface CardProps {
  title?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  variant?: 'default' | 'elevated' | 'outlined';
  className?: string;
}

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: React.ReactNode;
  footer?: React.ReactNode;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
  ariaLabel?: string;
}

export interface ToastProps {
  type?: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration?: number;
  onClose?: () => void;
  className?: string;
}

export interface ToastContextType {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;
}

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
  duration?: number;
}

export interface BadgeProps {
  label: string;
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
  icon?: React.ReactNode;
  className?: string;
}

export interface PillProps {
  label: string;
  onRemove?: () => void;
  variant?: 'default' | 'primary' | 'secondary';
  className?: string;
  ariaLabel?: string;
}

// ============================================================================
// Complex Component Types
// ============================================================================

export interface DataColumn<T> {
  key: keyof T;
  label: string;
  sortable?: boolean;
  width?: string;
  render?: (value: any, row: T) => React.ReactNode;
}

export interface DataTableProps<T> {
  columns: DataColumn<T>[];
  data: T[];
  onSort?: (column: keyof T, direction: 'asc' | 'desc') => void;
  onFilter?: (filters: Record<string, any>) => void;
  pageSize?: number;
  currentPage?: number;
  onPageChange?: (page: number) => void;
  loading?: boolean;
  className?: string;
}

export interface FormField {
  name: string;
  type: 'text' | 'email' | 'password' | 'number' | 'select' | 'checkbox' | 'textarea' | 'date';
  label: string;
  placeholder?: string;
  required?: boolean;
  options?: SelectOption[];
  validation?: (value: any) => string | undefined;
}

export interface FormBuilderProps {
  schema: FormField[];
  onSubmit: (data: Record<string, any>) => void | Promise<void>;
  defaultValues?: Record<string, any>;
  submitLabel?: string;
  loading?: boolean;
  className?: string;
}

export interface TimelineEvent {
  id: string;
  timestamp: Date;
  title: string;
  description?: string;
  type?: 'info' | 'success' | 'warning' | 'error';
  icon?: React.ReactNode;
  link?: {
    href: string;
    label: string;
  };
}

export interface TimelineProps {
  events: TimelineEvent[];
  onEventClick?: (event: TimelineEvent) => void;
  variant?: 'vertical' | 'horizontal';
  className?: string;
}

// ============================================================================
// Page Component Types
// ============================================================================

export interface WorkPackage {
  id: string;
  title: string;
  status: 'planned' | 'in_progress' | 'completed' | 'blocked';
  priority: 'low' | 'medium' | 'high' | 'critical';
  assignee?: string;
  dueDate?: Date;
  completedDate?: Date;
}

export interface DashboardStats {
  total: number;
  completed: number;
  inProgress: number;
  blocked: number;
}

export interface DashboardProps {
  workPackages: WorkPackage[];
  stats: DashboardStats;
  onWorkPackageSelect?: (wp: WorkPackage) => void;
  loading?: boolean;
}

export interface SettingsConfig {
  theme: 'light' | 'dark' | 'auto';
  notificationsEnabled: boolean;
  emailDigest: boolean;
  language: string;
}

export interface SettingsProps {
  config: SettingsConfig;
  onSave: (config: SettingsConfig) => void | Promise<void>;
  loading?: boolean;
}

export interface EvidenceItem {
  id: string;
  type: 'log' | 'screenshot' | 'video' | 'document';
  title: string;
  url: string;
  thumbnail?: string;
  timestamp: Date;
  testName?: string;
}

export interface EvidenceGalleryProps {
  items: EvidenceItem[];
  onItemSelect?: (item: EvidenceItem) => void;
  loading?: boolean;
}
