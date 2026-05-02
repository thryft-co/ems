import React from "react";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { Eye, Edit, Trash2, Calendar, DollarSign, User, Package, Users, FileText, MapPin, Truck, CreditCard } from "lucide-react";
import { OrderDetailResponse } from "@/features/orders/types/order";

interface OrderCardProps {
  order: OrderDetailResponse;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

interface DetailItem { label: string; value: string; icon: React.ReactNode; }

const statusConfig: Record<string, { label: string; variant: "default" | "success" | "warning" | "destructive" | "secondary" }> = {
  draft: { label: "Draft", variant: "secondary" },
  submitted: { label: "Submitted", variant: "default" },
  approved: { label: "Approved", variant: "success" },
  fulfilled: { label: "Fulfilled", variant: "success" },
  partially_fulfilled: { label: "Partial", variant: "warning" },
  cancelled: { label: "Cancelled", variant: "destructive" },
  paid: { label: "Paid", variant: "success" },
};

const OrderCard: React.FC<OrderCardProps> = ({ order, onView, onEdit, onDelete }) => {
  const status = statusConfig[order.status] || { label: order.status, variant: "secondary" as const };

  const getOrderTypeIcon = () => {
    switch (order.order_type) {
      case "customer_order": return <User className="h-4 w-4" />;
      case "purchase_order": return <Package className="h-4 w-4" />;
      case "distributor_order": return <Users className="h-4 w-4" />;
      default: return <FileText className="h-4 w-4" />;
    }
  };

  const getOrderTypeLabel = () => {
    switch (order.order_type) {
      case "customer_order": return "Customer";
      case "purchase_order": return "Purchase";
      case "distributor_order": return "Distributor";
      default: return order.order_type;
    }
  };

  const getSpecificDetails = (): DetailItem[] => {
    switch (order.order_type) {
      case "customer_order": return [
        { label: "Ref", value: order.customer_order?.customer_reference || "N/A", icon: <FileText className="h-3.5 w-3.5 text-muted-foreground/50" /> },
        { label: "Ship", value: order.customer_order?.shipping_method || "N/A", icon: <Truck className="h-3.5 w-3.5 text-muted-foreground/50" /> },
      ];
      case "purchase_order": return [
        { label: "Ref", value: order.purchase_order?.vendor_reference || "N/A", icon: <FileText className="h-3.5 w-3.5 text-muted-foreground/50" /> },
        { label: "Terms", value: order.purchase_order?.payment_terms || "N/A", icon: <CreditCard className="h-3.5 w-3.5 text-muted-foreground/50" /> },
      ];
      case "distributor_order": return [
        { label: "Territory", value: order.distributor_order?.territory || "N/A", icon: <MapPin className="h-3.5 w-3.5 text-muted-foreground/50" /> },
        { label: "Commission", value: order.distributor_order?.commission_rate ? `${order.distributor_order.commission_rate}%` : "N/A", icon: <DollarSign className="h-3.5 w-3.5 text-muted-foreground/50" /> },
      ];
      default: return [];
    }
  };

  const specificDetails = getSpecificDetails();
  const orderDate = order.order_date ? new Date(order.order_date).toLocaleDateString() : "N/A";

  return (
    <div className="bg-card rounded-2xl border-[0.5px] border-border/60 shadow-soft p-4 flex flex-col hover:shadow-soft-md transition-all duration-200 animate-fade-up">
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-[8px] bg-secondary/60 flex items-center justify-center flex-shrink-0">
            {getOrderTypeIcon()}
          </div>
          <div>
            <p className="text-[15px] font-semibold text-foreground leading-tight">{order.order_number}</p>
            <p className="text-[12px] text-muted-foreground/60">{getOrderTypeLabel()}</p>
          </div>
        </div>
        <Badge variant={status.variant}>{status.label}</Badge>
      </div>

      {/* Key info */}
      <div className="grid grid-cols-2 gap-x-3 gap-y-2 text-[13px] mb-3">
        <div className="flex items-center gap-2">
          <Calendar className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
          <span className="text-foreground/80 font-medium">{orderDate}</span>
        </div>
        <div className="flex items-center gap-2">
          <DollarSign className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
          <span className="text-foreground/80 font-medium">${order.total_amount.toLocaleString()}</span>
        </div>
        {specificDetails.map((detail, i) => (
          <div key={i} className="flex items-center gap-2 overflow-hidden">
            {detail.icon}
            <span className="text-foreground/80 font-medium truncate">{detail.value}</span>
          </div>
        ))}
        {order.items && order.items.length > 0 && (
          <div className="flex items-center gap-2">
            <Package className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
            <span className="text-foreground/80 font-medium">{order.items.length} items</span>
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 pt-3 border-t-[0.5px] border-border/40 mt-auto">
        {onView && <Button variant="ghost" size="sm" className="flex-1 h-9 text-[13px]" onClick={() => onView(order.id)}><Eye className="h-3.5 w-3.5 mr-1" />View</Button>}
        {onEdit && <Button variant="ghost" size="sm" className="flex-1 h-9 text-[13px]" onClick={() => onEdit(order.id)}><Edit className="h-3.5 w-3.5 mr-1" />Edit</Button>}
        {onDelete && <Button variant="ghost" size="sm" className="h-9 text-[13px] text-destructive hover:bg-destructive/8" onClick={() => onDelete(order.id)}><Trash2 className="h-3.5 w-3.5" /></Button>}
      </div>
    </div>
  );
};

export default OrderCard;
